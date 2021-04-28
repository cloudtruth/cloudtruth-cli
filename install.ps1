### Detection     ############################################################

### Arguments     ############################################################

param($version="", $url="", $authToken="", [Int16] $dryRun=0)

### Prerequisites ############################################################

### Auto-Version  ############################################################

if ($version -eq "") {
    $version = ((Invoke-WebRequest https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest).Content | ConvertFrom-Json).tag_name
    Write-Host  "Latest version: $version"
} else {
    Write-Host "Using version: $version"
}

if ($url -eq "") {
    $url = "https://github.com/cloudtruth/cloudtruth-cli/releases/download/$version"
}

# start off with empty additional headers, and add auth headers if needed
$headers=@{}
if ($authToken -eq "") {
    $headers=@{ Authorization="token $authToken" }
}

### Install-ish   ############################################################

# TODO, this puts cloudtruth.exe into $ENV:TEMP and leaves gunk behind

$tmp = New-TemporaryFile
$tmp = "$tmp.zip"
$out = "$tmp.out"
$package_base = "cloudtruth-$version-x86_64-pc-windows-msvc"
$full_url="$url/$package_base.zip"
if ($full_url.StartsWith("file://")) {
    $local = $full_url.Replace("file:", "")
    Copy-Item -Path $local -Destination $tmp
} else {
    Invoke-WebRequest -OutFile $tmp -Headers $headers "$full_url"
}

# make sure the file exists, and is bigger than 100 bytes
if (!(Test-Path $tmp -PathType Leaf) -or ((Get-Item $tmp).Length -lt 100)) {
    Write-Error "Failed to download: $full_url"
    return
}
Write-Host "Downloaded: $full_url"
$tmp | Expand-Archive -DestinationPath $out
if ($dryRun -ne 0) {
    "Skipping install of $package_base\cloudtruth.exe"
} else {
    Copy-Item -Path "$out\$package_base\cloudtruth.exe" -Destination $ENV:TEMP
}

if ($dryRun -eq 0) {
    & "$ENV:TEMP\cloudtruth.exe" --version
}
