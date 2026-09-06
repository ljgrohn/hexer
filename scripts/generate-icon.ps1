param(
    [string] $OutputDirectory = (Join-Path $PSScriptRoot "..\assets")
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null

function New-HexerBitmap([int] $Size) {
    $bitmap = [System.Drawing.Bitmap]::new(
        $Size,
        $Size,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear([System.Drawing.Color]::Transparent)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    $scale = $Size / 256.0
    $x = 16.0 * $scale
    $y = 16.0 * $scale
    $side = 224.0 * $scale
    $diameter = 108.0 * $scale
    $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
    $path.AddArc($x, $y, $diameter, $diameter, 180, 90)
    $path.AddArc($x + $side - $diameter, $y, $diameter, $diameter, 270, 90)
    $path.AddArc($x + $side - $diameter, $y + $side - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($x, $y + $side - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()

    $blue = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 108, 166, 255))
    $ink = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 16, 17, 21))
    $graphics.FillPath($blue, $path)
    $graphics.FillRectangle($ink, 72.0 * $scale, 64.0 * $scale, 26.0 * $scale, 128.0 * $scale)
    $graphics.FillRectangle($ink, 158.0 * $scale, 64.0 * $scale, 26.0 * $scale, 128.0 * $scale)
    $graphics.FillRectangle($ink, 98.0 * $scale, 115.0 * $scale, 60.0 * $scale, 25.0 * $scale)

    $ink.Dispose()
    $blue.Dispose()
    $path.Dispose()
    $graphics.Dispose()
    return $bitmap
}

function ConvertTo-PngBytes([System.Drawing.Bitmap] $Bitmap) {
    $stream = [System.IO.MemoryStream]::new()
    $Bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $stream.ToArray()
    $stream.Dispose()
    return $bytes
}

$preview = New-HexerBitmap 256
$preview.Save(
    (Join-Path $outputPath "hexer.png"),
    [System.Drawing.Imaging.ImageFormat]::Png
)
$preview.Dispose()

$entries = foreach ($size in @(16, 20, 24, 32, 40, 48, 64, 128, 256)) {
    $bitmap = New-HexerBitmap $size
    $data = ConvertTo-PngBytes $bitmap
    $bitmap.Dispose()
    [pscustomobject]@{ Size = $size; Data = $data }
}

$iconPath = Join-Path $outputPath "hexer.ico"
$stream = [System.IO.File]::Create($iconPath)
$writer = [System.IO.BinaryWriter]::new($stream)
$writer.Write([uint16] 0)
$writer.Write([uint16] 1)
$writer.Write([uint16] $entries.Count)

$offset = 6 + (16 * $entries.Count)
foreach ($entry in $entries) {
    $dimension = if ($entry.Size -eq 256) { 0 } else { $entry.Size }
    $writer.Write([byte] $dimension)
    $writer.Write([byte] $dimension)
    $writer.Write([byte] 0)
    $writer.Write([byte] 0)
    $writer.Write([uint16] 1)
    $writer.Write([uint16] 32)
    $writer.Write([uint32] $entry.Data.Length)
    $writer.Write([uint32] $offset)
    $offset += $entry.Data.Length
}

foreach ($entry in $entries) {
    $writer.Write([byte[]] $entry.Data)
}

$writer.Dispose()
$stream.Dispose()

Write-Host "Generated $iconPath and hexer.png"
