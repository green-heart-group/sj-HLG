param([ValidateSet("test","qa","release")][string]$Task="release")
$ErrorActionPreference="Stop"
$hlgRoot=Split-Path -Parent $PSScriptRoot
Set-Location $hlgRoot
$hlgInputs=@()
foreach($part in @("src","data","assets","native")){$hlgInputs+=Get-ChildItem (Join-Path $hlgRoot $part) -Recurse -File}
$hlgInputs+=Get-Item Cargo.toml,Cargo.lock,build.rs
$hlgLines=$hlgInputs | Sort-Object {$_.FullName.Substring($hlgRoot.Length).ToLowerInvariant()} | ForEach-Object {
  (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()+"  "+$_.FullName.Substring($hlgRoot.Length+1).Replace("\","/")
}
$hlgBytes=[Text.Encoding]::UTF8.GetBytes(($hlgLines -join "`n")+"`n")
$hlgHash=[Security.Cryptography.SHA256]::Create().ComputeHash($hlgBytes)
$env:HLG_BUILD_FINGERPRINT=([BitConverter]::ToString($hlgHash)).Replace("-","").ToLowerInvariant()
$env:HLG_QA_DIR=Join-Path $hlgRoot "work\qa-states"
if($Task -eq "test"){cargo test --locked --lib -- --nocapture}
elseif($Task -eq "qa"){cargo build --locked --release --features qa}
else{cargo build --locked --release}
exit $LASTEXITCODE
