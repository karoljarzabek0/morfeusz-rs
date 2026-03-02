# Configuration
$url = "http://download.sgjp.pl/morfeusz/20251116/polimorf-20251116.tab.gz"
$gzFile = Join-Path $PSScriptRoot "polimorf-20251116.tab.gz"
$outFile = Join-Path $PSScriptRoot "polimorf-20251116.tab"

try {
    # 1. Download
    Write-Host "Downloading PoliMorf dictionary..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $url -OutFile $gzFile

    # 2. Decompress using .NET
    Write-Host "Unpacking..." -ForegroundColor Cyan
    $input1 = [System.IO.File]::OpenRead($gzFile)
    $output = [System.IO.File]::Create($outFile)
    $gzipStream = New-Object System.IO.Compression.GzipStream($input1, [System.IO.Compression.CompressionMode]::Decompress)

    $gzipStream.CopyTo($output)

    # Explicitly close to release the files
    $gzipStream.Dispose()
    $output.Dispose()
    $input1.Dispose()

    # 3. Cleanup
    Write-Host "Cleaning up..." -ForegroundColor Cyan
    Remove-Item $gzFile

    Write-Host "Success! '$($outFile)' is ready." -ForegroundColor Green
}
catch {
    Write-Error "An error occurred: $_"
}