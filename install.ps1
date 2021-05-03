#
# Copyright (C) 2021 CloudTruth, Inc.
#
# NOTE: this puts cloudtruth.exe into $ENV:TEMP and leaves gunk behind
#       we'll make a chocolatey or scoop package soon, hopefully
#

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
        HelpMessage="Do not install, just download.")]
    [Switch]
    $dryRun,

    [Parameter(
        ParameterSetName="Testing",
        HelpMessage="A GitHub Draft Release ID for integration testing.")]
    [String]
    $releaseId
)

### Detection     ############################################################

# TODO: we  only support x86_64 platform right now
$platform ="x86_64"

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

$tmp = New-TemporaryFile
$tmp = "$tmp.zip"
$out = "$tmp.out"
$package = "cloudtruth-$version-$platform-pc-windows-msvc"

if (!$releaseId) {
    # normal production codepath
    $base_url = "https://github.com/cloudtruth/cloudtruth-cli/releases/download"
    $download_url = "${base_url}/${version}/${package}.zip"
} else {
    # handle a GitHub draft release for integration testing
    $content = (Invoke-WebRequest -Headers @{Authorization = "token $authToken"} "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/${releaseId}/assets").Content
    $assets = $content | ConvertFrom-Json
    $assetId = ($assets | where { $_.Name -eq "${package}.zip" }).id
    $download_url = "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/assets/${assetId}"
}

Invoke-WebRequest -OutFile $tmp -Headers $headers "${download_url}"
Write-Host "Downloaded: $package"

$tmp | Expand-Archive -DestinationPath $out
if ($dryRun.IsPresent) {
    "Skipping install of ${package}\cloudtruth.exe"
} else {
    Copy-Item -Path "$out\${package}\cloudtruth.exe" -Destination $ENV:TEMP
}

if (!$dryRun.IsPresent) {
    & "$ENV:TEMP\cloudtruth.exe" --version
}

return "$ENV:TEMP\cloudtruth.exe"
