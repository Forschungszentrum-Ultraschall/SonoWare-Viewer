let channel = 0;
let global_header;
let a_scan_x;
let a_scan_y;
let a_scan_handler;
let single_view_handler;
let multi_view_handler_left;
let multi_view_handler_right;
let time;
let file_name

Chart.defaults.font.size = 16;

let color_mapping = color_fz_u;

/**
 * Get the Jet color code for a specified value
 * @param {Number} value Number in range [0..1]
 * @returns HSL representation for value on the Jet colormap 
 */
function jet_color_map(value) {
    hue = 240;
    saturation = 0;
    lightness = 0;

    switch(true) {
        case value < 0.05:
            saturation = 100 * value * 20;
            lightness = 50 * value * 20;
            break;
        case value < 0.5:
            hue = 240 - 120 * (value - 0.05) / 0.45;
            saturation = 100
            lightness = 50 + 25 * (value - 0.05) / 0.45;
            break;
        case value < 0.95:
            hue = 120 - 120 * (value - 0.5) / 0.45;
            saturation = 100
            lightness = 75 - 25 * (value - 0.5) / 0.45;
            break;
        default:
            hue = 0;
            saturation = 100 - 100 * (value - 0.95) * 20;
            lightness = 50 + 50 * (value - 0.95) * 20;
            break;
    }

    return `hsl(${hue},${saturation}%,${lightness}%)`;
}

/**
 * Get a gray-scale RGB code for a given value
 * @param {Number} value number in range [0..1]
 * @returns RGB code for a gray-scale color
 */
function black_white(value) {
    const rgb_value = (value * 255).toFixed(0);
    return `rgb(${rgb_value}, ${rgb_value}, ${rgb_value})`;
}

/**
 * Get the HSL value of the FZ-U (blue scale) colormap
 * @param {Number} value number in range [0..1]
 * @returns HSL representation of value for the FZ-U colormap
 */
function color_fz_u(value) {
    let lightness = 0;
    let saturation = 0;

    switch(true) {
        case value < 0.25:
            saturation = 81;
            lightness = 23 * value * 4;
            break;
        case value < 0.5:
            saturation = 81;
            lightness = 23 + 16 * (value - 0.25) * 4;
            break;
        case value < 0.75:
            saturation = 81;
            lightness = 39 + 9 * (value - 0.5) * 4;
            break;
        default:
            saturation = 81 * (1 - (value - 0.75) * 4);
            lightness = 48 + 52 * (value - 0.75) * 4;
            break;
    }

    return `hsl(210, ${saturation}%, ${lightness}%)`;
}

/**
 * Get the HSL representation of a value for a BLUE-WHITE-RED colormap
 * @param {Number} value number in range [0..1]
 * @returns HSL color code for the specified value
 */
function red_white_blue(value) {
    const color_value = value <= 0.5 ? 240 : 0;

    return `hsl(${color_value}, ${Math.abs(100 - 200 * value)}%, ${100 - Math.abs(70 - 140 * value)}%)`;
}

/**
 * Get the HSL representation of a value for a colormap with multiple color steps
 * @param {Number} value number in range [0..1]
 * @returns HSL color code for the specified value
 */
function stairs(value) {
    let hue = 0;
    let saturation = 0;
    let lightness = 0;

    switch(true) {
        case value < (1 / 16):
            hue = 0;
            saturation = 82 * value * 16;
            lightness = 33 * value * 16;
            break;
        case value < (2 / 16): 
            hue = 0;
            saturation = 82 - 29 * (value - 1 / 16) * 16;
            lightness = 33 + 20 * (value - 1 / 16) * 16; 
            break;
        case value < (3 / 16): 
            hue = 0;
            saturation = 53 + 29 * (value - 2 / 16) * 16;
            lightness = 53 + 26 * (value - 2 / 16) * 16;
            break;
        case value < (4 / 16):
            hue = 30 * (value - 3 / 16) * 16;
            saturation = 82 - 9 * (value - 3 / 16) * 16;
            lightness = 79 - 42 * (value - 3 / 16) * 16;
            break;
        case value < (5 / 16): 
            hue = 30;
            saturation = 73 - 17 * (value - 4 / 16) * 16;
            lightness = 37 + 21 * (value - 4 / 16) * 16;
            break;
        case value < (6 / 16): 
            hue = 30;
            saturation = 56 + 44 * (value - 5 / 16) * 16;
            lightness = 58 + 26 * (value - 5 / 16) * 16;
            break;
        case value < (7 / 16): 
            hue = 30 + 50 * (value - 6 / 16) * 16;
            saturation = 100 - 35 * (value - 6 / 16) * 16;
            lightness = 84 - 44 * (value - 6 / 16) * 16;
            break;
        case value < (8 / 16): 
            hue = 80
            saturation = 65 - 5 * (value - 7 / 16) * 16;
            lightness = 40 + 23 * (value - 7 / 16) * 16;
            break;
        case value < (9 / 16): 
            hue = 80 + 44 * (value - 8 / 16) * 16;
            saturation = 60 - 25 * (value - 8 / 16) * 16;
            lightness = 63 + 22 * (value - 8 / 16) * 16;
            break;
        case value < (10 / 16):
            hue = 124 + 76 * (value - 9 / 16) * 16;
            saturation = 35 + 24 * (value - 9 / 16) * 16;
            lightness = 75 - 30 * (value - 9 / 16) * 16;
            break;
        case value < (11 / 16): 
            hue = 200;
            saturation = 59 + 6 * (value - 10 / 16) * 16;
            lightness = 45 + 23 * (value - 10 / 16) * 16;
            break;
        case value < (12 / 16): 
            hue = 200 + 38 * (value - 11 / 16) * 16;
            saturation = 65 - 21 * (value - 11 / 16) * 16;
            lightness = 68 - 13 * (value - 11 / 16) * 16;
            break;
        case value < (13 / 16): 
            hue = 238 + 12 * (value - 12 / 16) * 16;
            saturation = 44 + 8 * (value - 12 / 16) * 16;
            lightness = 55 - 6 * (value - 12 / 16) * 16;
            break;
        case value < (14 / 16): 
            hue = 250;
            saturation = 52 + 19 * (value - 13 / 16) * 16;
            lightness = 49 + 24 * (value - 13 / 16) * 16;
            break;
        case value < (15 / 16): 
            hue = 250;
            saturation = 71 + 29 * (value - 14 / 16) * 16;
            lightness = 73 + 12 * (value - 14 / 16) * 16;
            break;
        default: 
            hue = 250;
            saturation = 100 - 100 * (value - 15 / 16) * 16;
            lightness = 85 + 15 * (value - 15 / 16) * 16;
            break;
    }

    return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
}

/**
 * Get the color code of the HOT colormap for a value 
 * @param {Number} value number in range [0..1]
 * @returns HSL value for the specified value
 */
function hot(value) {
    let saturation = 100;
    let lightness = 0;

    switch(true) {
        case value < 0.11:
            saturation = 94 * value / 0.11;
            lightness = 18 * value * 9;
            break;
        case value < 0.22:
            saturation = 94 - 7 * (value - 0.11) / 0.11;
            lightness = 18 + 3 * (value - 0.11) * 9;
            break;
        case value < 0.33:
            saturation = 87 - 24 * (value - 0.22) / 0.11;
            lightness = 21 + 10 * (value - 0.22) * 9;
            break;
        case value < 0.44:
            saturation = 63 - 17 * (value - 0.33) / 0.11;
            lightness = 31 + 12 * (value - 0.33) * 9;
            break;
        case value < 0.55:
            saturation = 46 + 34 * (value - 0.44) / 0.11;
            lightness = 43 - 4 * (value - 0.44) * 9;
            break;
        case value < 0.66:
            saturation = 80 + 5 * (value - 0.55) / 0.11;
            lightness = 39 + 3 * (value - 0.55) * 9;
            break;
        case value < 0.77:
            saturation = 85 - 16 * (value - 0.66) / 0.11;
            lightness = 42 + 7 * (value - 0.66) * 9;
            break;
        case value < 0.88:
            saturation = 69 + 3 * (value - 0.77) / 0.11;
            lightness = 49 + 16 * (value - 0.77) * 9;
            break;
        default:
            saturation = 72 + 28 * value;
            lightness = 65 + 35 * (value - 0.88) * 9
            break;
    }

    return `hsl(${76 - 166 * (1 - value)}, ${saturation}%, ${lightness}%)`;
}
