#
# Copyright (C) 2021 CloudTruth, Inc.
#
# NOTE: this puts cloudtruth.exe into $ENV:TEMP and leaves gunk behind
#       we'll make a chocolatey or scoop package soon, hopefully
#

### Detection     ############################################################

### Arguments     ############################################################

Param(
    [Parameter(
        HelpMessage="Specify the version of the CLI to install.  If not specified, the latest version is installed.")]
    [String]
    $Version,

    [Parameter(
        ParameterSetName="Testing",
        HelpMessage="A GitHub authentication token for integration testing.")]
    [String]
    $authToken,

    [Parameter(
        ParameterSetName="Testing",
        HelpMessage="A GitHub Draft Release ID for integration testing.")]
    [String]
    $releaseId,
)

### Prerequisites ############################################################

### Auto-Version  ############################################################

if ($version -eq "") {
    $version = ((Invoke-WebRequest https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest).Content | ConvertFrom-Json).tag_name
    Write-Host "Latest version: $version"
} else {
    Write-Host "Using version: $version"
}

$headers = @{Accept = "application/octet-stream"}
if ($authToken) {
    $headers += @{Authorization = "token $authToken"}
}

### Install-ish   ############################################################

# TODO, this puts cloudtruth.exe into $ENV:TEMP and leaves gunk behind

if ($authToken) {

$tmp = New-TemporaryFile
$tmp = "$tmp.zip"
$out = "$tmp.out"
$package_base = "cloudtruth-$version-x86_64-pc-windows-msvc"
$full_url="$url/$package_base.zip"
if ($full_url.StartsWith("file://")) {
    $local = $full_url.Replace("file://", "").Replace("/", "\")
    Copy-Item -Path $local -Destination $tmp
} else {
    Invoke-WebRequest -OutFile $tmp -Headers $headers "$full_url"
}

# make sure the file exists, and is bigger than 100 bytes
if (!(Test-Path $tmp -PathType Leaf)) {
    Write-Error "Failed to download: $full_url"
    return
}
$filesize = (Get-Item $tmp).Length
if ($filesize -lt 100) {
    Write-Error "Problem downloading: $full_url"
    Write-Error "File exists, but is only $filesize bytes"
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
