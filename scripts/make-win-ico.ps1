<#
.SYNOPSIS
Rebuilds src-tauri/icons/icon.ico from a square source PNG.

.DESCRIPTION
`tauri icon` stores every ICO frame as a PNG payload. The Windows shell
surfaces that read the exe icon resource (Task Manager, alt-tab, Explorer)
handle PNG frames inconsistently, so the small sizes end up scaled from the
wrong frame and look squashed. This writes the small frames as uncompressed
32bpp DIBs and keeps PNG for the 256px frame only. 32x32 is written first
because tauri-codegen takes the default window icon from the first entry.

Frames up to $SmallMaxSize come from the head-only artwork: the full mark is
1.27:1 wide, so at 16px the headphone band and cups collapse into the face.
Both sources live in designs/ and are derived from src/assets/IMG/Logo_Fav.svg.

.EXAMPLE
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/make-win-ico.ps1
#>
param(
  [string]$SourcePath = "designs/app-icon.png",
  [string]$SmallSourcePath = "designs/app-icon-small.png",
  [int]$SmallMaxSize = 24,
  [string]$OutputPath = "src-tauri/icons/icon.ico"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

# 32 first: tauri-codegen uses entries()[0] as the default window icon.
$dibSizes = @(32, 16, 20, 24, 40, 48, 64, 96, 128)
$pngSize = 256

function Get-Resized([System.Drawing.Image]$image, [int]$size) {
  $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $g.DrawImage($image, 0, 0, $size, $size)
  $g.Dispose()
  return $bmp
}

# BITMAPINFOHEADER + bottom-up BGRA rows + 1bpp AND mask, the layout RT_ICON expects.
function Get-DibPayload([System.Drawing.Bitmap]$bmp) {
  $size = $bmp.Width
  $maskStride = [int][Math]::Floor(($size + 31) / 32) * 4
  $stream = New-Object System.IO.MemoryStream
  $writer = New-Object System.IO.BinaryWriter $stream

  $writer.Write([uint32]40)                                          # biSize
  $writer.Write([int32]$size)                                        # biWidth
  $writer.Write([int32]($size * 2))                                  # biHeight: colour rows + mask rows
  $writer.Write([uint16]1)                                           # biPlanes
  $writer.Write([uint16]32)                                          # biBitCount
  $writer.Write([uint32]0)                                           # biCompression = BI_RGB
  $writer.Write([uint32](($size * $size * 4) + ($maskStride * $size))) # biSizeImage
  $writer.Write([int32]0)                                            # biXPelsPerMeter
  $writer.Write([int32]0)                                            # biYPelsPerMeter
  $writer.Write([uint32]0)                                           # biClrUsed
  $writer.Write([uint32]0)                                           # biClrImportant

  for ($y = $size - 1; $y -ge 0; $y--) {
    for ($x = 0; $x -lt $size; $x++) {
      $p = $bmp.GetPixel($x, $y)
      $writer.Write([byte]$p.B)
      $writer.Write([byte]$p.G)
      $writer.Write([byte]$p.R)
      $writer.Write([byte]$p.A)
    }
  }

  for ($y = $size - 1; $y -ge 0; $y--) {
    $row = New-Object byte[] $maskStride
    for ($x = 0; $x -lt $size; $x++) {
      if ($bmp.GetPixel($x, $y).A -lt 128) {
        $index = [int][Math]::Floor($x / 8)
        $row[$index] = $row[$index] -bor (0x80 -shr ($x % 8))
      }
    }
    $writer.Write($row, 0, $row.Length)
  }

  $writer.Flush()
  $payload = $stream.ToArray()
  $writer.Dispose()
  return ,$payload
}

function Get-PngPayload([System.Drawing.Bitmap]$bmp) {
  $stream = New-Object System.IO.MemoryStream
  $bmp.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
  $payload = $stream.ToArray()
  $stream.Dispose()
  return ,$payload
}

$sourceImage = [System.Drawing.Image]::FromFile((Resolve-Path $SourcePath).Path)
$smallImage = [System.Drawing.Image]::FromFile((Resolve-Path $SmallSourcePath).Path)
$frames = New-Object System.Collections.ArrayList

foreach ($size in $dibSizes) {
  $art = if ($size -le $SmallMaxSize) { $smallImage } else { $sourceImage }
  $bmp = Get-Resized $art $size
  [void]$frames.Add([pscustomobject]@{ Size = $size; Payload = [byte[]](Get-DibPayload $bmp) })
  $bmp.Dispose()
}
$bmp = Get-Resized $sourceImage $pngSize
[void]$frames.Add([pscustomobject]@{ Size = $pngSize; Payload = [byte[]](Get-PngPayload $bmp) })
$bmp.Dispose()
$smallImage.Dispose()
$sourceImage.Dispose()

$out = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter $out
$writer.Write([uint16]0)                # reserved
$writer.Write([uint16]1)                # resource type: icon
$writer.Write([uint16]$frames.Count)

$offset = 6 + (16 * $frames.Count)
foreach ($frame in $frames) {
  # 256px is encoded as 0 in the directory entry.
  $dimension = if ($frame.Size -ge 256) { 0 } else { $frame.Size }
  $writer.Write([byte]$dimension)
  $writer.Write([byte]$dimension)
  $writer.Write([byte]0)                # palette entries
  $writer.Write([byte]0)                # reserved
  $writer.Write([uint16]1)              # colour planes
  $writer.Write([uint16]32)             # bits per pixel
  $writer.Write([uint32]$frame.Payload.Length)
  $writer.Write([uint32]$offset)
  $offset += $frame.Payload.Length
}
foreach ($frame in $frames) {
  $payload = [byte[]]$frame.Payload
  $writer.Write($payload, 0, $payload.Length)
}
$writer.Flush()
$bytes = $out.ToArray()
$writer.Dispose()

$target = Join-Path (Get-Location).Path $OutputPath
[System.IO.File]::WriteAllBytes($target, $bytes)
Write-Host ("Wrote {0} ({1} bytes, {2} frames: {3})" -f $OutputPath, $bytes.Length, $frames.Count, (($frames | ForEach-Object { $_.Size }) -join ", "))
