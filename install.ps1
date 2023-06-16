#
# Copyright (C) 2021 CloudTruth, Inc.
#
# Downloads the CloudTruth CLI executable and returns a path to it.
# You can Invoke-Expression on the result to run CloudTruth
# or ampersand notation, for example:
#
# $ctExe = .\install.ps1
# (& $ctExe --version)
#
# To see any informational messages, set $InformationPreference = 'Continue'
# before running.
# 
# NOTE: this puts cloudtruth.exe into Temp:\ and leaves gunk behind
#       we'll make a chocolatey or scoop package soon, hopefully
#

### Arguments     ############################################################

Param(
    [Parameter(
        HelpMessage="Specify the version of the CLI to install.  If not specified, the latest version is installed.")]
    [String]
    $version,

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

# TODO: we only support 64-bit x86 and ARM
if ($env:PROCESSOR_ARCHITECTURE -eq "AMD64") {
    $platform ="x86_64"
} elseif ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    $platform = "aarch64"
} else {
    throw "Unsupported architecture: ${env:PROCESSOR_ARCHITECTURE}"
}

### Prerequisites ############################################################



### Auto-Version  ############################################################

if ($version -eq "") {
    $version = ((Invoke-WebRequest https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest).Content | ConvertFrom-Json).tag_name
    Write-Information -MessageData "Latest version: $version"
} else {
    Write-Information -MessageData "Using version: $version"
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
Write-Information -MessageData "Downloaded: $package"

$tmp | Expand-Archive -DestinationPath $out

if ($dryRun.IsPresent) {
    Write-Warning -MessageData "Skipping install of ${package}\cloudtruth.exe"
} else {
    # see: https://github.com/PowerShell/PowerShell/issues/4216
    # $ENV:TEMP was not cross-platform and is now replaced with Temp:\ PSDrive
    Copy-Item -Path "$out\${package}\cloudtruth.exe" -Destination "Temp:\"
    Write-Information -MessageData (& "Temp:\cloudtruth.exe" --version)
    return "Temp:\cloudtruth.exe"
}
