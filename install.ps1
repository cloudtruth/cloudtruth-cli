### Detection     ############################################################

### Arguments     ############################################################

# TODO

### Prerequisites ############################################################

### Auto-Version  ############################################################

$latest = ((Invoke-WebRequest https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest).Content | ConvertFrom-Json).tag_name

### Install-ish   ############################################################

# TODO, this puts cloudtruth.exe into $ENV:TEMP and leaves gunk behind

$tmp = New-TemporaryFile
$tmp = "$tmp.zip"
$out = "$tmp.out"
Invoke-WebRequest -OutFile $tmp https://github.com/cloudtruth/cloudtruth-cli/releases/download/$latest/cloudtruth-$latest-x86_64-pc-windows-msvc.zip
$tmp | Expand-Archive -DestinationPath $out
Copy-Item -Path "$out\cloudtruth-$latest-x86_64-pc-windows-msvc\cloudtruth.exe" -Destination $ENV:TEMP

& "$ENV:TEMP\cloudtruth.exe" --version
