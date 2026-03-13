Add-Type -AssemblyName System.Drawing

$iconsDir = "C:\lex-exploratory\Aether\crates\aether-desktop\src-tauri\icons"

# Create 32x32 bitmap with a lightning bolt
$bmp = New-Object System.Drawing.Bitmap(32, 32)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(80, 80, 120))
$pen = New-Object System.Drawing.Pen([System.Drawing.Color]::Gold, 3)
$points = @(
    (New-Object System.Drawing.Point(18, 2)),
    (New-Object System.Drawing.Point(10, 14)),
    (New-Object System.Drawing.Point(16, 14)),
    (New-Object System.Drawing.Point(12, 30)),
    (New-Object System.Drawing.Point(22, 16)),
    (New-Object System.Drawing.Point(16, 16))
)
$g.DrawLines($pen, $points)

# Save 32x32
$bmp.Save("$iconsDir\32x32.png", [System.Drawing.Imaging.ImageFormat]::Png)

# Create 128x128
$bmp128 = New-Object System.Drawing.Bitmap(128, 128)
$g128 = [System.Drawing.Graphics]::FromImage($bmp128)
$g128.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g128.DrawImage($bmp, 0, 0, 128, 128)
$bmp128.Save("$iconsDir\128x128.png", [System.Drawing.Imaging.ImageFormat]::Png)

# Create 256x256 (for @2x)
$bmp256 = New-Object System.Drawing.Bitmap(256, 256)
$g256 = [System.Drawing.Graphics]::FromImage($bmp256)
$g256.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$g256.DrawImage($bmp, 0, 0, 256, 256)
$bmp256.Save("$iconsDir\128x128@2x.png", [System.Drawing.Imaging.ImageFormat]::Png)

# Create ICO
$icon = [System.Drawing.Icon]::FromHandle($bmp128.GetHicon())
$fileStream = [System.IO.File]::Create("$iconsDir\icon.ico")
$icon.Save($fileStream)
$fileStream.Close()

# Copy for tray and macOS
Copy-Item "$iconsDir\128x128.png" "$iconsDir\icon.png"
Copy-Item "$iconsDir\icon.ico" "$iconsDir\icon.icns" -ErrorAction SilentlyContinue

Write-Host "Icons created successfully in $iconsDir"
Get-ChildItem $iconsDir
