#!/usr/bin/env bash
# Quick script to create a simple icon for yoinkctl

mkdir -p assets

# Create SVG icon
cat > assets/yoinkctl.svg <<'EOF'
<svg width="256" height="256" xmlns="http://www.w3.org/2000/svg">
  <!-- Background gradient -->
  <defs>
    <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#6366f1;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#8b5cf6;stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <!-- Rounded square background -->
  <rect width="256" height="256" rx="56" fill="url(#grad)"/>
  
  <!-- Eyedropper icon -->
  <g transform="translate(128, 128)">
    <!-- Dropper body -->
    <path d="M -20,-40 L -20,-10 L -10,0 L 10,0 L 20,-10 L 20,-40 Z" 
          fill="white" stroke="none"/>
    
    <!-- Drop -->
    <ellipse cx="0" cy="20" rx="12" ry="15" fill="white" opacity="0.9"/>
    
    <!-- Color preview circle inside dropper -->
    <circle cx="0" cy="-25" r="10" fill="#22c55e"/>
  </g>
</svg>
EOF

echo "✓ Created assets/yoinkctl.svg"

# Convert to PNG if imagemagick/inkscape is available
if command -v convert >/dev/null 2>&1; then
    convert -background none assets/yoinkctl.svg assets/yoinkctl.png
    echo "✓ Converted to assets/yoinkctl.png (using ImageMagick)"
elif command -v inkscape >/dev/null 2>&1; then
    inkscape assets/yoinkctl.svg --export-filename=assets/yoinkctl.png -w 256 -h 256
    echo "✓ Converted to assets/yoinkctl.png (using Inkscape)"
else
    echo "⚠️  Install ImageMagick or Inkscape to convert SVG to PNG"
    echo "   Or convert manually at: https://cloudconvert.com/svg-to-png"
fi