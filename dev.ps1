#!/usr/bin/env pwsh

$initialLocation = (Get-Location).Path

function Exit-Script {
    param ($code, $message)
    if ($message) {
        Write-Host $message
    }
    Set-Location "$initialLocation"
    Exit $code
}

if (!($IsWindows)) {
    Exit-Script 0 "Please use the Bash script 'dev' instead."
}

if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
    $arch = "arm64"
    $dotnetArch = "arm64"
} else {
    $arch = "amd64"
    $dotnetArch = "x64"
}

$workspace = $PSScriptRoot
$dockerfile = "Dockerfile"
$dockerImg = "etscript/dev:r1.73-n8.0"

Set-Location $workspace

Get-Command docker *> $null
if (!($?)) {
    Exit-Script 1 "Error: could not find the 'docker' command."
}

if (!(Test-Path "$workspace/$dockerfile" -PathType Leaf)) {
    Exit-Script 1 "Error: could not find '$dockerfile'."
}

docker image inspect $dockerImg *> $null
if (!($?)) {
    Write-Host "Using '$dockerfile' to build an image..."
    docker buildx build `
        --platform linux/$arch `
        --build-arg dotnet_arch=$dotnetArch `
        -t $dockerImg `
        -f "$workspace/$dockerfile" `
        "$workspace"
}

docker run -it --init --rm `
    -u etscript:etscript `
    --mount "type=bind,source=$workspace,target=/workspace" `
    -w /workspace `
    --cap-add SYS_PTRACE `
    --security-opt seccomp=unconfined `
    $dockerImg
