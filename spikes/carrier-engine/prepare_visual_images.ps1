param([string]$OutputDirectory = 'D:/Dev/nemu-wt/P0-12/reports/P0-12/inputs/images')
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Preserve previous images' }
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$fontPath = 'C:/Windows/Fonts/msyh.ttc'
$fonts = [Drawing.Text.PrivateFontCollection]::new()
$fonts.AddFontFile($fontPath)
$records = @()
try {
    foreach ($size in @(256, 512, 1024)) {
        foreach ($kind in @('red', 'green', 'blue', 'geometry', 'zh', 'en', 'ja')) {
            $bitmap = [Drawing.Bitmap]::new($size, $size)
            $graphics = [Drawing.Graphics]::FromImage($bitmap)
            $font = [Drawing.Font]::new($fonts.Families[0], [single]($size / 16), [Drawing.FontStyle]::Regular, [Drawing.GraphicsUnit]::Pixel)
            $text = ''
            try {
                $graphics.Clear([Drawing.Color]::White)
                $graphics.TextRenderingHint = [Drawing.Text.TextRenderingHint]::AntiAliasGridFit
                switch ($kind) {
                    'red' { $graphics.Clear([Drawing.Color]::Red) }
                    'green' { $graphics.Clear([Drawing.Color]::Lime) }
                    'blue' { $graphics.Clear([Drawing.Color]::Blue) }
                    'geometry' {
                        $graphics.FillRectangle([Drawing.Brushes]::Red, [single]($size/8), [single]($size/8), [single]($size/4), [single]($size/4))
                        $graphics.FillEllipse([Drawing.Brushes]::Blue, [single]($size*5/8), [single]($size/8), [single]($size/4), [single]($size/4))
                        $points = [Drawing.PointF[]]@([Drawing.PointF]::new($size/2, $size/2), [Drawing.PointF]::new($size/4, $size*7/8), [Drawing.PointF]::new($size*3/4, $size*7/8))
                        $graphics.FillPolygon([Drawing.Brushes]::Green, $points)
                    }
                    'zh' { $text = "今天晴天`n红色方块`n合成测试 123" }
                    'en' { $text = "Sunny day`nRed square`nSynthetic 123" }
                    'ja' { $text = "今日は晴れ`n赤い四角`n合成テスト 123" }
                }
                if ($text) { $graphics.DrawString($text, $font, [Drawing.Brushes]::Black, [single]($size/16), [single]($size/4)) }
                $path = Join-Path $OutputDirectory "$kind-$size.png"
                $bitmap.Save($path, [Drawing.Imaging.ImageFormat]::Png)
                $records += [ordered]@{file="$kind-$size.png"; width=$size; height=$size; kind=$kind; text=$text; sha256=(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()}
            } finally { $font.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }
        }
    }
} finally { $fonts.Dispose() }
[ordered]@{generator='System.Drawing on Windows'; font=$fontPath; fontSha256=(Get-FileHash -LiteralPath $fontPath).Hash.ToLowerInvariant(); synthetic=$true; images=$records} |
    ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'manifest.json') -Encoding utf8NoBOM
Write-Output "Generated $($records.Count) synthetic PNG images"
