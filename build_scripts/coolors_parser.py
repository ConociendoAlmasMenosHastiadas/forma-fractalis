#!/usr/bin/env python3
"""
Coolors.co Palette Parser

Converts palette XML from coolors.co into forma-fractalis JSON colormap format.

Usage:
    python coolors_parser.py < palette.xml > colormap.json
    python coolors_parser.py -i palette.xml -o colormap.json
"""

import sys
import json
import xml.etree.ElementTree as ET
import argparse


def parse_coolors_xml(xml_content):
    """Parse coolors.co XML palette format.
    
    Args:
        xml_content: String containing XML palette data
        
    Returns:
        List of color dictionaries with name, hex, and RGB values
    """
    root = ET.fromstring(xml_content)
    
    colors = []
    for color_elem in root.findall('color'):
        color = {
            'name': color_elem.get('name'),
            'hex': color_elem.get('hex'),
            'r': int(color_elem.get('r')),
            'g': int(color_elem.get('g')),
            'b': int(color_elem.get('b'))
        }
        colors.append(color)
    
    return colors


def create_colormap_json(colors, name=None):
    """Create forma-fractalis colormap JSON from color list.
    
    Args:
        colors: List of color dictionaries
        name: Optional name for the colormap (defaults to first color name)
        
    Returns:
        Dictionary in forma-fractalis colormap format
    """
    if not colors:
        raise ValueError("No colors provided")
    
    # Generate colormap name from colors if not provided
    if name is None:
        # Use the first two color names to create a descriptive name
        if len(colors) >= 2:
            name = f"{colors[0]['name']}_{colors[-1]['name']}"
        else:
            name = colors[0]['name']
    
    # Create gradient stops in the correct format for forma-fractalis
    stops = []
    num_colors = len(colors)
    
    for i, color in enumerate(colors):
        # Evenly distribute colors across the 0.0-1.0 range
        position = i / (num_colors - 1) if num_colors > 1 else 0.0
        
        stops.append({
            "position": round(position, 4),
            "color": {
                "r": color['r'],
                "g": color['g'],
                "b": color['b']
            }
        })
    
    # Create the colormap structure
    colormap = {
        "name": name.replace(' ', '_').replace('-', '_').lower(),
        "stops": stops
    }
    
    return colormap


def main():
    parser = argparse.ArgumentParser(
        description='Convert coolors.co palette XML to forma-fractalis JSON colormap'
    )
    parser.add_argument('-i', '--input', 
                       help='Input XML file (default: stdin)',
                       type=argparse.FileType('r'),
                       default=sys.stdin)
    parser.add_argument('-o', '--output',
                       help='Output JSON file (default: stdout)',
                       type=argparse.FileType('w'),
                       default=sys.stdout)
    parser.add_argument('-n', '--name',
                       help='Colormap name (default: derived from color names)',
                       type=str)
    parser.add_argument('--pretty',
                       help='Pretty-print JSON output',
                       action='store_true')
    
    args = parser.parse_args()
    
    try:
        # Read XML content
        xml_content = args.input.read()
        
        # Parse colors
        colors = parse_coolors_xml(xml_content)
        
        if not colors:
            print("Error: No colors found in input", file=sys.stderr)
            sys.exit(1)
        
        # Create colormap
        colormap = create_colormap_json(colors, args.name)
        
        # Output JSON
        if args.pretty:
            json.dump(colormap, args.output, indent=2)
        else:
            json.dump(colormap, args.output)
        
        args.output.write('\n')
        
        # Print summary to stderr (so it doesn't interfere with piped output)
        if args.output != sys.stdout:
            print(f"✓ Created colormap '{colormap['name']}' with {len(colors)} colors", 
                  file=sys.stderr)
    
    except ET.ParseError as e:
        print(f"Error parsing XML: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
