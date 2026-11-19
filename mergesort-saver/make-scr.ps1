# Build the screensaver and copy it as mergesort.scr next to this script.
# Then right-click mergesort.scr in Explorer and choose Install.
cargo build --release -p mergesort-saver --manifest-path "$PSScriptRoot\..\Cargo.toml"
if ($?) {
    Copy-Item "$PSScriptRoot\..\target\release\mergesort-saver.exe" "$PSScriptRoot\mergesort.scr"
    Write-Host "Made $PSScriptRoot\mergesort.scr - right-click it and choose Install."
}
