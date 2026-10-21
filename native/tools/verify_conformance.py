"""HLG source boundary gate. Standard-library only; never alters source materials."""
from pathlib import Path
import argparse,collections,csv,hashlib,json,re,xml.etree.ElementTree as ET,zipfile
from datetime import datetime,timezone

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def fingerprint(root):
    paths=[p for part in ("src","data","assets","native") for p in (root/part).rglob("*") if p.is_file()]
    paths += [root/n for n in ("Cargo.toml","Cargo.lock","build.rs")]
    paths.sort(key=lambda p:p.relative_to(root).as_posix().lower())
    text="".join(f"{sha(p)}  {p.relative_to(root).as_posix()}\n" for p in paths)
    return hashlib.sha256(text.encode()).hexdigest()
def read_json(p):return json.loads(p.read_text(encoding="utf-8-sig"))
def extract_doc(path):
    ns={"w":"http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
    with zipfile.ZipFile(path) as z:root=ET.fromstring(z.read("word/document.xml"))
    text=["".join(t.text or "" for t in p.findall(".//w:t",ns)) for p in root.findall(".//w:p",ns)]
    return [t for t in text if t.strip()]
def original_achievements(lines):
    out=[]
    for i in range(2083,2784):
        text=lines[i];m=re.fullmatch(r"([&@%~+#]*)(【(Root|成就)：(.+)】)",text)
        if m:out.append({"name":m[4],"root":m[3]=="Root","revision_marker":m[1],"lines":[]})
        elif out:out[-1]["lines"].append(text)
    for a in out:
        fields={};key=None
        for t in a.pop("lines"):
            m=re.match(r"^(成就图标|文本内容|达成条件|前置成就)：(.*)",t)
            if m:key=m[1];fields[key]=[m[2]] if m[2] else []
            elif t.startswith(("*","彩蛋成就","成就边框","弹出提示")):key=None
            elif key:fields[key].append(t)
        for output,key in [("icon","成就图标"),("text","文本内容"),("condition","达成条件"),("parent","前置成就")]:
            a[output]="\n".join(fields.get(key,[]))
    return out
def original_nodes(lines):
    out=[]
    for ln,line in enumerate(lines,1):
        if not (line.startswith("【") and re.search(r"】\(",line)) and "【化学办公间】" not in line:continue
        for name,raw in re.findall(r"【([^】]+)】\(([^)]+)\)",line):
            x,y,f=raw.split(",",2);x=int(x);y=int(y)
            floors=range(int(f[1]),int(f[3])+1) if f.startswith("{") else (range(1,6) if f=="F" else [int(re.sub(r"\D","",f))])
            for z in floors:
                if x==7 and y==2:z={"物理实验室":2,"电学实验室":3,"生物实验室":4}.get(name,z)
                if name=="生物办公室":x=2
                out.append((name,raw,ln,x,y,z))
    return collections.Counter(out)
def write_csv(path,columns,rows):
    with path.open("w",encoding="utf-8-sig",newline="") as f:
        writer=csv.DictWriter(f,fieldnames=columns);writer.writeheader();writer.writerows(rows)
def run(args):
    source=args.source.resolve();evidence=args.evidence.resolve();out=args.output.resolve();out.mkdir(parents=True,exist_ok=True)
    checks=[]
    def check(key,ok,detail):checks.append({"id":key,"result":"PASS" if ok else "FAIL","detail":detail})
    identity=fingerprint(source)
    authority=read_json(source/"data/authority.json")
    for name,meta in authority["sources"].items():
        p=args.style_source if name=="京工文集.docx" else args.materials/name
        check("source:"+name,p.is_file() and sha(p)==meta["sha256"],meta)
    lines=extract_doc(args.materials/"理恐MC-策划案（修订版） 2022.9.29.docx")
    expected=original_achievements(lines);actual=read_json(source/"data/achievements.json")
    check("achievement_count",len(actual)==101 and sum(not a["root"] for a in actual)==99,{"entries":len(actual),"non_root":sum(not a["root"] for a in actual)})
    compared=["name","root","revision_marker","icon","text","condition","parent"]
    exact=len(actual)==len(expected) and all(all(a[k]==e[k] for k in compared) for a,e in zip(actual,expected))
    check("achievement_verbatim",exact,"All seven fields compared directly with the original DOCX paragraphs; no whitespace normalization.")
    check("achievement_text_hashes",all(hashlib.sha256(a["text"].encode()).hexdigest()==a["text_sha256"] for a in actual),"SHA-256 of each description")
    m=read_json(source/"data/map.json");nodes=m["nodes"];surface=[n for n in nodes if n["worlds"]&1]
    observed=collections.Counter((n["name"],n["raw_coordinate"],n["source_line"],n["x"],n["y"],n["z"]) for n in surface)
    expected_nodes=original_nodes((args.materials/"理工恐怖故事.txt").read_text(encoding="utf-8-sig").splitlines())
    check("surface_coordinates",observed==expected_nodes and len(surface)==68,{"surface_nodes":len(surface),"missing":list((expected_nodes-observed).elements()),"extra":list((observed-expected_nodes).elements())})
    check("edge_authority",all(e["source"] in ["TXT","TXT_GRID","SKETCH","MC"] and e["rationale"] for e in m["edges"]),dict(collections.Counter(e["source"] for e in m["edges"])))
    items=read_json(source/"data/items.json");known=set(authority["user_authorizations"])
    check("item_authority",all(x["source"] and x["effect"] and set(x["implementation_authority"])<=known for x in items),{"items":len(items),"meaning":"source and adaptation declarations; not a claim that every optional branch was played"})
    support=set(read_json(source/"data/achievement_support.json"));names={a["name"] for a in actual}
    check("achievement_support_names",support<=names,{"wired_non_root":sum(a["name"] in support and not a["root"] for a in actual),"unopened_non_root":sum(a["name"] not in support and not a["root"] for a in actual)})
    runs=read_json(evidence/"full-runs-v2.json")
    check("public_routes",len(runs)>=6 and all(r["public_actions_only"] and r["relics"]==8 and r["ending"]=="循环" and r["echo_rooms"]==1101 and r["echo_revisions"]>1 for r in runs),runs)
    check("route_source_identity",all(r["build_fingerprint"]==identity for r in runs),identity)
    log=(evidence/"test.stdout.log").read_text(encoding="utf-8-sig")
    match=re.search(r"test result: ok\. (\d+) passed; 0 failed",log)
    check("engine_tests",bool(match) and int(match[1])>=19,{"passed":int(match[1]) if match else 0,"includes":"seed connectivity, repeated reconfiguration, cards, rejection, item consumption, character, endings, persistence"})
    native=read_json(evidence/"native-verification-v2.json")
    required={"movement_reaches_selected_coordinate","support_discard_and_replenish","lamp_actually_repels_wxh","reshape_changes_actual_graph","reshape_keeps_current_room","reshape_receipt_names_changed_routes","procedural_save_round_trip"}
    check("native_interaction",native.get("result")=="PASS" and required<=set(native.get("checks",[])),{"checks":len(native.get("checks",[])),"input":native.get("input")})
    check("native_source_identity",native.get("build_fingerprint")==identity,identity)
    prod=read_json(evidence/"production-startup.json")
    check("production_binary",prod.get("result")=="PASS" and prod.get("qa_feature") is False and prod.get("os")=="windows" and prod.get("build_fingerprint")==identity,prod)
    visual=read_json(evidence/"visual-review.json")
    check("visual_review",visual.get("result")=="PASS" and visual.get("build_fingerprint")==identity and len(visual.get("reviewed_screenshots",[]))>=8,visual)
    reviewed=visual.get("reviewed_screenshots",[])
    check("visual_file_identity",bool(reviewed) and all(
        r["file"].removeprefix("v2-").removesuffix(".png") in native.get("screenshots",[]) and
        (evidence.parent/"screenshots"/r["file"]).is_file() and
        sha(evidence.parent/"screenshots"/r["file"])==r["sha256"]
        for r in reviewed),"Reviewed PNG hashes match the exported native screenshot set.")
    scope=read_json(source/"docs/coverage.json")
    check("coverage_disclosure",bool(scope) and all(x.get("authority") and x.get("status") in ["implemented","partial","not_implemented","interpretation"] for x in scope),"Scope gaps remain visible; a passing gate does not make them implemented.")
    check("unmapped_additions",all(x.get("authority")!="UNAUTHORIZED" for x in scope),"Manual source/adaptation ledger reviewed; structural checks cannot prove semantic fidelity by themselves.")
    achrows=[{"name":a["name"],"state":"trigger_wired" if a["name"] in support else "not_open","description":a["text"],"condition":a["condition"],"parent":a["parent"],"source_paragraph":a["source_start"],"description_sha256":a["text_sha256"]} for a in actual if not a["root"]]
    write_csv(out/"成就逐项对照.csv",list(achrows[0]),achrows)
    write_csv(out/"道具逐项对照.csv",["id","name","kind","source","effect","adaptation_authority"],[{"id":i["id"],"name":i["name"],"kind":i["kind"],"source":i["source"],"effect":i["effect"],"adaptation_authority":",".join(i["implementation_authority"])} for i in items])
    write_csv(out/"表世界坐标.csv",["id","name","x","y","z","raw_coordinate","source","correction"],[{k:n[k] for k in ["id","name","x","y","z","raw_coordinate","source","correction"]} for n in surface])
    write_csv(out/"地图连接依据.csv",["a","b","worlds","source","rationale"],[dict(e,a=nodes[e["a"]]["name"]+"@"+nodes[e["a"]]["id"],b=nodes[e["b"]]["name"]+"@"+nodes[e["b"]]["id"]) for e in m["edges"]])
    report={"checked_utc":datetime.now(timezone.utc).isoformat(),"source_fingerprint":identity,"scope_gate":"PASS" if all(c["result"]=="PASS" for c in checks) else "FAIL","coverage":"PARTIAL_WITH_DECLARED_GAPS","checks":checks,"coverage_ledger":scope}
    (out/"GATE_REPORT.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"scope_gate":report["scope_gate"],"coverage":report["coverage"],"checks":len(checks),"failed":[c["id"] for c in checks if c["result"]=="FAIL"],"fingerprint":identity},ensure_ascii=False))
    return 0 if report["scope_gate"]=="PASS" else 1
if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--source",type=Path,default=Path(__file__).resolve().parent.parent)
    p.add_argument("--materials",type=Path,required=True);p.add_argument("--style-source",type=Path,required=True)
    p.add_argument("--evidence",type=Path,required=True);p.add_argument("--output",type=Path,required=True)
    raise SystemExit(run(p.parse_args()))
