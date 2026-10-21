use crate::{cards::*, content::*, game::*};
impl Game {
    pub fn rescue_key(&self) -> Option<&'static str> {
        let key = match NODES[self.current].id.as_str() {
            "saponify" if self.flag("soap_story") => "soap_student",
            "garden" if self.flag("met_ll") && self.flag("met_ljx") => "floating_student",
            "abyss" => "buried_student",
            _ => return None,
        };
        if self.flag(&format!("saved_{key}")) {
            None
        } else {
            Some(key)
        }
    }
    pub fn quest_cards(&self) -> Vec<Card> {
        let mut v = vec![];
        let mut add =
            |id: &str, title: &str, story: &str, rule: &str, ok: bool, why: &str, source: &str| {
                v.push(
                    Card::new(
                        Action::Quest(id.into()),
                        title,
                        story,
                        rule,
                        "一回合；除明确标注外无需掷骰",
                        source,
                    )
                    .require(ok, why),
                );
            };
        match NODES[self.current].id.as_str(){
            "clock" if self.flag("copies_seen")&&!self.flag("saved_clock_student")=>add("save_clock","除去学长头顶的复制体","你先分清那两副叠在一起的面孔，等上面的东西抬头，再把攻击对准不属于学长的那一部分。","d6 + 洞察 ≥ 8；成功只击除复制体，学长脱困并留下生魂之泪；失败可重试。",true,"","MC:1943-1956;U08,U09"),
            "reception"=>{
                if !self.flag("guide"){add("guide","阅读离校须知","纸上依次列出八门课程，收纳盒中的空格与它们恰好相对。你把要求抄到本子上，等值班人员回来时，再问问为什么会有这样的规矩。","记下八信物目标；下一站可先去钟楼取用学长留下的材料。",true,"","MC:15,171-178");}
                add("deliver","将信物放入回收盒","你按课程顺序清点背包，再把手伸向窗口后面的透明收纳盒。","首次交付触发喷泉剧情并返回大厅；再次交付八件信物才开门。",self.relic_count()==8,"需要同时携带八件信物。","MC:171-178");
            }
            "gate" if self.flag("gate_open")=>add("leave","迈出校门","伸缩门终于留出一道足以让人通过的空隙，门外的街灯仍亮着。你最后看了一眼背包里留下的东西，慢慢走了出去。","按已作出的选择判定六种原案结局；七滴生魂之泪具有最高优先级。",true,"","MC:1993-2074"),
            "lockers"=>{
                if !self.flag("locker_loot")||!self.has(Item::Lamp)&&(!self.has(Item::Ink)||!self.has(Item::Matches)||!self.has(Item::Solution)) {
                    add("lockers","翻开学长留的柜子","柜门里夹着一张受潮的纸，几件实验用品压在下面。它们被收拾得并不漂亮，但每一件都像是特意为有人再次打开柜门而留下的。","取得空墨水瓶、未知溶液、安全火柴、空笔记本；收录酒精灯手记。",true,"","MC:33-40,1327-1351");
                }
            }
            "fountain" if !self.flag("mirror")=>add("mirror","等倒影先抬起手","你没有立刻离开水边，而是试着让自己的动作慢下来，看倒影会在什么时候追上。","确认表里入口；喷泉、钟楼、化学办公室和复制长廊可使用穿镜卡。",true,"","U02"),
            "chem_office" if !self.has(Item::Card)=>add("card","拿起波波的门卡","门卡压着备课本的一角，你原本只想借它进教室，拿走作业便回家。纸页被带动的那一刻，走廊里的灯同时暗了一下。","取得门卡；首次会被送到化学实验室并遭遇 WXH。",true,"","MC:74-82"),
            "classroom" if !self.has(Item::Wenjin)=>add("wenjin","拿回问津作业","作业本正放在你记得的位置，旁边却多出一本不属于你的手记。你把它们一起收进书包，窗外的风声便忽然变得清楚起来。","取得问津和学长手记·壹；LL 会现身，先持握问津再应对。",self.has(Item::Card),"需要携带波波的门卡。","MC:28-32,667-676"),
            "chinese_office" if !self.clues.contains("diary3")=>add("diary3","查看书架里的手记","笔记本被塞在两本教辅中间，第一页已经折了角，像是有人读到这里时不得不停下来。","收录学长手记·叁，补全关于老师与学生的线索。",true,"","MC:41-45,1352-1362"),
            "company" if !self.flag("company_loot")=>add("company","打开书架后的箱子","柜里有没拆的食品和几件舞台道具，最下面压着学长写的字条。你先读完字条，再挑了眼下能带走的东西。","取得铁剑、戏服盔甲、牛奶与饼干；收录学长手记·肆与拾叁。",true,"","MC:230-240,1113-1125"),
            "storage" if self.turn>=self.counter("storage_next")=>add("storage","登记领取试剂","你照着柜门上的领用单核对物品，把能辨认的几瓶单独放好。别人的签名不知为何都十分相似，你还是认真写下了自己的名字。","取得 3 燃油、1 未知溶液、1 不明溶液。12 回合后可补领。",self.flag("met_wxh"),"先触发 WXH 剧情。","MC:89-94;MC:4 主线材料再生"),
            "chem_inner" if !self.has(Item::Mold)=>add("mold","查看隐藏的整理箱","箱子藏在洗手池下面，盖上的锁扣却没有扣好。你拿起模具时，想起主任检查学生作业的习惯，便又把原处的物品摆回了整齐的样子。","取得制皂模具、不明溶液；成品仍需脂肪。",true,"","MC:1231-1243"),
            "physics_office" if !self.has(Item::Meter)=>add("parts","收集电表零件","零件盒中有几样东西在示意图上被圈了出来。你把导线绕开、按颜色分好，等着看看这份说明最后究竟要接通什么。","取得表头、矢量箭头、鳄鱼夹和导线；每种缺少时补齐一件。",true,"","MC:870-898,1139-1147"),
            "physics_lab"=>{
                if !self.has(Item::Thunder){add("thunder","打开闪着电光的箱子","箱里的器物比想象中轻，搭扣却被一道细小的电弧牢牢连着。有人在门后说，先拿着吧，想弄明白它的用处，还得再翻几页笔记。","取得怒霆雷跃；收录工整的笔记·卷一、卷二，并遭遇 ZYY。",true,"","MC:125-130,688-699");}
            }
            "garden"=>{
                if !self.has(Item::Hook){add("hook","收起水边的引魂钓","线头正在水里轻轻移动，你顺着它看向假山的方向，听见一个很低的声音催你把竿拿稳。","取得引魂钓与 4 黏痰；开启地牢线索。",true,"","MC:131-140");}
                if self.turn>=self.counter("leaf_next"){add("leaf","接住飘下的槐叶","树梢落下一片带着金光的叶子，落到掌心后还留着温度。你把它夹好，抬头时，综合楼顶层恰有一抹轮形火光闪过。","取得金色槐叶；叶片每 12 回合重新飘落一次。",true,"","MC:102-111,1245-1250;U09 时间换算");}
            }
            "bio_office" if !self.flag("mj_hint")=>add("mj","听完门后的交谈","门后的人说得很客气，称呼也像白天办公时一样熟稔。你站在门外，本来打算询问门卡的去处，听到后半段时却忘了敲门。","得到复制长廊线索；在里世界寻找课程主任办公室。",true,"","MC:46-64,379-395"),
            "director" if !self.clues.contains("memo")=>add("memo","翻阅课程主任备忘录","桌上没有你要找的门卡，备忘录倒摊在最常查阅的一页。你越往下读，越觉得登记表上的分类，并没有把你与样本区分开来。","收录备忘录，触发被同化者；击败可取得脂肪。",true,"","MC:59-64,1448-1481"),
            "specimen" if !self.has(Item::Diploid)=>add("diploid","空手取出标本","你把手里的东西先收好，按照柜边留下的空隙伸进去。玻璃映着自己的脸，柜里那团干瘪的东西却让你迟迟不愿确认它的形状。","取得凋亡的二倍体并遭遇复制体。",self.equip.is_none(),"先在背包腾出双手，原案要求空手拾取。","MC:65-73,708-715"),
            "geo_office"=>{
                if self.turn>=self.counter("ore_next"){add("ore","搬下工位旁的矿料","老师把煤块叫作午餐，却没有阻止你查看标本盒里的矿石。炉火映过来时，有一块晶体短暂亮了一下。","取得 3 整块煤炭和 1 翡渊晶矿；12 回合后可补领。",true,"","MC:144-160;MC:4");}
            }
            "training" if !self.has(Item::Wheel)=>add("wheel","取下法台上的骨轮","器物并没有想象中烫，靠近时听到的细小声音却始终没有停止。你抬起它，背后的日光灯便一盏接一盏亮了起来，照出课桌底下许多空着的位置。","取得炼火骨轮；HJF 现身。用骨轮飞行需要燃油和一处楼顶。",true,"","MC:119-124,732-739"),
            "saponify" if !self.has(Item::Soap)=>add("soap_trial","按记录完成皂化实验","主任的记录写得相当详细，失败原因和返工步骤都在同一页。你已经见过整理箱中的成品，却还不知道轮到自己时，最后会留下怎样的一块。","本次角色死亡并在池畔重生；d7 = 7 得细研香皂，其余得精制肥皂。剧情、成长与信物保留。",self.flag("soap_story"),"先以粗制肥皂驱离 WXH。","MC:426-445（详细机制）"),
            "principal" if !self.has(Item::Domain)=>add("domain","触及黑域祭坛","桌后的空间看不清边界，你只能沿着指尖碰到的地方慢慢往前。墙上的题词隐约亮过一次，像是在核对你是否真的带齐了该带的东西。","d6 + 定力 ≥ 8；成功取得浮寰黑域，失败损失 8 理智并可重试。",RELICS.iter().take(7).all(|&i|self.has(i)),"需要其余七件信物。","MC:161-170;U09"),
            "rehearsal" if !self.has(Item::Spear)&&!self.has(Item::ThunderSpear)=>add("spear","搬起排练间的铁枪","铁枪远比舞台上看起来沉，你把尾端抵在地面上歇了片刻，才重新找好双手的位置。","取得铁质花枪；可正面攻击，亦可配合充能雷跃。",true,"","TXT:138-139"),
            "teacher_wc"=>add("slug","按下水龙头旁的按钮","按钮装在一个不太方便的位置，外面还贴着电气设备的检修条。你犹豫了一会儿，仍按了下去。","承受 d6 点生命伤害，取得 3 黏痰。",true,"","MC:241-245"),
            "yawwai"=>{
                if !self.flag("spark_taken")&&self.counter("leaf_saves")>=3&&self.flag("flown"){add("spark","接受桌上的火花","那个声音把火光称作礼物，让你把一路所见都记下来。桌上的空位像是早已安排好，只等你把自己的笔记放上去。","取得灵魂火花；可与空笔记本合成你的手记。",true,"","MC:749-776");}
                if !self.flag("blood_taken")&&self.counter("copy_deaths")>=3&&self.flag("wore_diploid"){add("blood","拿起密封的样品瓶","桌边没有别人，劝你放下疑问的声音却始终没有停。你握住那只样品瓶时，忽然觉得自己早就知道它会放在哪里。","取得愚者之血；可与空笔记本合成虚无的造物。",true,"","MC:807-823");}
            }
            _=>{}
        }
        if self.world == World::Echo
            && [
                "garden", "clock", "basement", "training", "martial", "canteen", "abyss",
            ]
            .contains(&NODES[self.current].id.as_str())
            && !self.flag(&format!("altar_{}", NODES[self.current].id))
        {
            add("altar","破开环绕的金轮","那团光被一圈金轮罩着，仍有细小的人声从里面传来。你握稳能用的器物，试着从轮光转过的间隙接近它。","d6 + 体魄 ≥ 7；成功破除本处祭坛，并掷 d7：7 时掉落生魂之泪。每座祭坛只破除一次。",true,"","MC:270-278;U09");
        }
        if self.rescue_key().is_some() {
            v.push(
                Card::new(
                    Action::Use(Item::Hook),
                    "用引魂钓接住学长",
                    "你把线甩向那只伸过来的手，放慢动作，等对方重新找到一点可以支撑的地方。",
                    "d6 + 机敏 + 2 ≥ 7；成功获得 1 生魂之泪，失败可再试。",
                    "1 体力；一回合",
                    "MC:1943-1989;U08,U09",
                )
                .item(Item::Hook)
                .require(self.has(Item::Hook), "需要引魂钓。"),
            );
        }
        v
    }
    pub(crate) fn quest(&mut self, id: &str) {
        match id {
            "save_clock" => {
                self.spend(1);
                if self.test(
                    "辨认并击除复制体",
                    crate::character::Attribute::Insight,
                    8,
                    0,
                ) {
                    self.mark("saved_clock_student");
                    self.give(Item::Tear, 1);
                    self.unlock("引渡人");
                    self.note("复制体从学长头顶消散，留下 1 滴生魂之泪；学长已经离开。");
                } else {
                    self.sanity -= 2;
                    self.note("你没有找到安全的出手时机，理智 -2；没有误伤学长，可以重试。");
                }
            }
            "altar" => {
                self.spend(1);
                if self.test("破除祭坛", crate::character::Attribute::Body, 7, 0) {
                    self.mark(&format!("altar_{}", NODES[self.current].id));
                    let value = 1 + self.random(7) as i32;
                    self.last.rolls.push(Roll {
                        label: "祭坛生魂掉落".into(),
                        sides: 7,
                        value,
                        bonus: 0,
                        target: 7,
                        defense_sides: 0,
                        defense: 0,
                        success: value == 7,
                    });
                    if value == 7 {
                        self.give(Item::Tear, 1);
                        self.unlock("引渡人");
                        self.note("游魂飞散时留下 1 滴生魂之泪。");
                    } else {
                        self.note("祭坛已经破除，本次没有生魂之泪掉落。原案的该项概率为 1/7。");
                    }
                    if self
                        .flags
                        .iter()
                        .filter(|f| f.starts_with("altar_"))
                        .count()
                        == 7
                    {
                        self.unlock("残祀末飨");
                        for e in &mut self.enemies {
                            if e.kind == EnemyKind::Hjf {
                                e.stun = 8;
                            }
                        }
                        self.note("七座祭坛均已破除，HJF 的夺魄在接下来七回合内受到抑制。");
                    }
                } else {
                    self.sanity -= 2;
                    self.note("金轮仍在转动，理智 -2；祭坛未破，可以重试。");
                }
            }
            "guide" => {
                self.mark("guide");
                self.note(
                    "八件信物须在传达室交付。门卡在三层化学办公室；可先去钟楼柜群拿灯的材料。",
                );
            }
            "lockers" => {
                for i in [Item::Ink, Item::Matches, Item::Solution, Item::Notebook] {
                    if !self.has(i) {
                        self.give(i, 1)
                    }
                }
                self.document("lamp");
                self.mark("locker_loot");
                self.note("所需材料已放入背包。可选择制作酒精灯，材料配齐便确定完成。");
            }
            "mirror" => {
                self.mark("mirror");
                self.note("已确认镜面入口；在带双环记号的地点使用穿镜卡。");
            }
            "card" => {
                self.give(Item::Card, 1);
                self.unlock("取得门卡");
                if !self.flag("met_wxh") {
                    self.mark("met_wxh");
                    self.current = node("chem_lab");
                    self.world = World::Surface;
                    self.spawn(EnemyKind::Wxh);
                    self.unlock("暴露行踪");
                    self.note("门卡引发传送，你到了化学实验室。先持握酒精灯，再用灯火驱离；也可用移动卡离开。");
                }
            }
            "wenjin" => {
                self.give(Item::Wenjin, 1);
                self.document("diary1");
                self.mark("met_ll");
                self.unlock("“作业狂”");
                self.unlock("琴瑟和鸣");
                self.spawn(EnemyKind::Ll);
            }
            "diary3" => self.document("diary3"),
            "company" => {
                for (i, n) in [
                    (Item::Sword, 1),
                    (Item::Armor, 1),
                    (Item::Milk, 2),
                    (Item::Food, 3),
                ] {
                    self.give(i, n);
                }
                self.document("diary4");
                self.document("diary13");
                self.mark("company_loot");
                self.unlock("前人遗资");
            }
            "storage" => {
                self.give(Item::Fuel, 3);
                self.give(Item::Solution, 1);
                self.give(Item::Alkali, 1);
                self.counts.insert("storage_next".into(), self.turn + 12);
            }
            "mold" => {
                self.give(Item::Mold, 1);
                self.give(Item::Alkali, 2);
                self.unlock("喷香的皂盒");
            }
            "parts" => {
                for i in [Item::Gauge, Item::Arrow, Item::Clip, Item::Wire] {
                    if !self.has(i) {
                        self.give(i, 1)
                    }
                }
            }
            "thunder" => {
                self.give(Item::Thunder, 1);
                self.document("thunder1");
                self.document("thunder2");
                self.unlock("初逢仙圣");
                self.unlock("偶获雷具");
                self.spawn(EnemyKind::Zyy);
            }
            "hook" => {
                self.give(Item::Hook, 1);
                self.give(Item::Slug, 4);
                self.mark("met_ljx");
                self.unlock("炮火连天？");
            }
            "leaf" => {
                self.give(Item::Leaf, 1);
                self.counts.insert("leaf_next".into(), self.turn + 12);
                self.unlock("拾到槐叶");
            }
            "mj" => {
                self.mark("mj_hint");
                self.unlock("轻信于人");
                self.note("课程主任的线索指向三层复制长廊。那里有另一面的入口。");
            }
            "memo" => {
                self.document("memo");
                self.mark("copies_seen");
                self.unlock("突破重围");
                self.spawn(EnemyKind::Assimilated);
            }
            "diploid" => {
                self.give(Item::Diploid, 1);
                self.document("specimen");
                self.unlock("标本馆奇遇记");
                self.unlock("百躯取首");
                self.spawn(EnemyKind::Copy);
                self.enemies.push(Enemy {
                    kind: EnemyKind::Copy,
                    node: self.current,
                    world: self.world,
                    hp: 12,
                    stun: 1,
                    chasing: false,
                });
            }
            "ore" => {
                self.give(Item::Coal, 3);
                self.give(Item::Ore, 1);
                self.counts.insert("ore_next".into(), self.turn + 12);
                self.unlock("误陷怪屋");
            }
            "wheel" => {
                self.give(Item::Wheel, 1);
                self.unlock("“圣遗物”");
                self.spawn(EnemyKind::Hjf);
            }
            "soap_trial" => {
                self.hp = 0;
                self.rebirth("实验记录又多了一行", true);
            }
            "domain" => {
                if self.test("黑域中的触碰", crate::character::Attribute::Will, 8, 0) {
                    self.give(Item::Domain, 1);
                    self.document("motto");
                    self.unlock("深幽居处");
                } else {
                    self.sanity -= 8;
                    self.note("尚未找到祭坛的边界；理智损失 8，可再次尝试。");
                }
            }
            "spear" => {
                self.give(Item::Spear, 1);
                self.unlock("浑铁奇兵");
            }
            "slug" => {
                let dmg = 1 + self.random(6) as i32;
                self.hp -= dmg;
                self.give(Item::Slug, 3);
                self.note(format!("按钮带来电击：生命 -{dmg}；水龙头吐出 3 份黏痰。"));
            }
            "spark" => {
                self.mark("spark_taken");
                self.give(Item::Spark, 1);
                self.unlock("灵魂火花");
            }
            "blood" => {
                self.mark("blood_taken");
                self.give(Item::Blood, 1);
                self.unlock("愚者之血");
            }
            "deliver" => {
                if !self.flag("fountain_story") {
                    self.mark("fountain_story");
                    self.current = node("hall");
                    self.world = World::Surface;
                    self.last.major = true;
                    self.note("第一次触及回收盒，眼前的登记簿忽然变成了喷泉边的倒影。你回到综合楼大厅，八件信物仍在；再次前往传达室交付。");
                } else {
                    self.mark("gate_open");
                    self.note("回收盒核对了八件信物，校门已经打开。去校门口选择离校。");
                }
            }
            "leave" => {
                let ending = self.ending();
                self.status = Status::Ended(ending);
                self.last.major = true;
                self.note(format!("达成原案结局：{}。", ending.name()));
            }
            _ => {}
        }
        self.gain_once(&format!("quest_{id}"), 2);
    }
}
