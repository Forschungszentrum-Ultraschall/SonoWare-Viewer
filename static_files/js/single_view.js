const display_mode = document.getElementById('display_mode');
const points_scaling = document.getElementById('points');
const mm_scaling = document.getElementById('mm');
const window_start = document.getElementById('aperture_start');
const window_end = document.getElementById('aperture_end');
const color_map_selector = document.getElementById('colormap');
const color_bar = document.getElementById('colorbar');
const color_reset_button = document.getElementById('reset_color_border_button');
const display_mode_reset_button = document.getElementById('reset_display_mode_button');

const a_scan_rel = document.getElementById('a_scan_rel');
const a_scan_db = document.getElementById('a_scan_db');

const color_bar_min = document.getElementById('view_min');
const color_bar_max = document.getElementById('view_max');

const color_range_min = document.getElementById('color_border_min');
const color_range_max = document.getElementById('color_border_max');

let data_min = 0;
let data_max = 1;

window_start.value = "";
window_end.value = "";
color_range_max.value = "";
color_range_min.value = "";

let last_entry = 0;

let scale = points_scaling.value;
let a_scan_scale_x = 1;
let a_scan_scale_y = 1;

points_scaling.checked = true;
display_mode.value = "";

color_map_selector.value = 'fz-u';

a_scan_rel.addEventListener('click', (_) => {
    update_borders(true);
});

a_scan_db.addEventListener('click', (_) => {
    update_borders(true);
});

color_range_min.addEventListener('change', (event) => {
    let new_color_min = Number(event.target.value);
    
    if (new_color_min >= Number(color_range_max.value)) {
        new_color_min = Number(color_range_max.value) - 1;
    }

    event.target.value = new_color_min;
    set_border_value(color_bar_min, new_color_min);
});

color_range_max.addEventListener('change', (event) => {
    let new_color_max = Number(event.target.value);

    if (new_color_max <= Number(color_range_min.value)) {
        new_color_max = Number(color_range_min.value) + 1;
    }

    event.target.value = new_color_max;
    set_border_value(color_bar_max, new_color_max);
});

color_reset_button.addEventListener('click', (_) => {
    color_range_min.value = data_min === Math.trunc(data_min) ? data_min.toFixed(0) : data_min.toFixed(4);
    color_range_max.value = data_max === Math.trunc(data_max) ? data_max.toFixed(0) : data_max.toFixed(4);
    set_border_value(color_bar_min, data_min);
    set_border_value(color_bar_max, data_max);
});

color_map_selector.addEventListener('change', (event) => {
    color_bar.classList = '';
    color_bar.classList.add(event.target.value);

    update_color_mapping_function(event.target.value);

    update_borders();
});

function set_border_value(text_element, value) {
    if (value !== Math.floor(value)) {
        text_element.textContent = value.toFixed(4);
    }
    else {
        text_element.textContent = value.toFixed(0);
    }

    re_color();
}

function re_color() {
    if(single_view_handler !== undefined) {
        const range_max = Number(color_range_max.value);
        const range_min = Number(color_range_min.value);

        single_view_handler.data.datasets = single_view_handler.data.datasets.map(dataset => {
            const new_colors = [];

            dataset.data.forEach(data => {
                if(data.v !== undefined) {
                    let value;

                    if(data.v > range_max) {
                        value = 1;
                    }
                    else {
                        if(data.v < range_min) {
                            value = 0;
                        }
                        else {
                            value = (data.v - range_min) / (range_max - range_min);
                        }
                    }

                    new_colors.push(color_mapping(value));
                }
            });

            if(new_colors.length !== 0) {
                dataset.backgroundColor = new_colors;
            }

            return dataset;
        });

        single_view_handler.update();
    }
}

function update_color_mapping_function(function_request) {
    switch(function_request) {
        case 'black-white':
            color_mapping = black_white;
            break;
        case 'jet':
            color_mapping = jet_color_map;
            break;
        case 'red-white-blue':
            color_mapping = red_white_blue;
            break;
        case 'fz-u':
            color_mapping = color_fz_u;
            break;
        case 'hot':
            color_mapping = hot;
            break;
        case 'stairs':
            color_mapping = stairs;
            break;
    }
}

color_bar.addEventListener('mousemove', (event) => {
    const rel_target = 1 - event.offsetY / color_bar.offsetHeight;
    color_bar.title = (Number(color_range_min.value) + (Number(color_range_max.value) - Number(color_range_min.value)) * rel_target).toFixed(4);
});

display_mode.addEventListener('change',(_) => {
    update_borders(true);
});

window_start.addEventListener('change', (_) => {
    update_borders(true);
});

window_end.addEventListener('change', (_) => {
    update_borders(true);
});

points_scaling.addEventListener('click', (event) => {
    if(scale !== event.target.value) {
        scale = event.target.value;
        a_scan_scale_x = 1;
        a_scan_scale_y = 1;
        update_axis_scaling(1 / global_header.res_x, 1 / global_header.res_y, true, 'Punkte');
    }
});

mm_scaling.addEventListener('click', (event) => {
    if(scale !== event.target.value) {
        scale = event.target.value;
        a_scan_scale_x = global_header.res_x;
        a_scan_scale_y = global_header.res_y;
        update_axis_scaling(global_header.res_x, global_header.res_y, true, 'mm');
    }
});

display_mode_reset_button.addEventListener('click', (_) => {
    window_start.value = time[0].toFixed(4);
    window_end.value = time.slice(-1)[0].toFixed(4);

    if (single_view_handler !== undefined) {
        single_view_handler.destroy();
        single_view_handler = undefined;
    }
    
    update_borders(true);
});

function update_borders(new_mode) {
    let borders = get_window_borders();

    switch(display_mode.value) {
        case 'c-scan':
            load_c_scan(channel, borders[0], borders[1], new_mode);
            break;
        case 'd-scan':
            load_d_scan(channel, borders[0], borders[1], new_mode);
            break;
    }
}

function update_axis_scaling(x_scaling, y_scaling, update_axis_max, axis_label) {
    if (single_view_handler !== undefined) {
        single_view_handler.data.datasets = single_view_handler.data.datasets.map(entry => {
            entry.data = entry.data.map(data_entry => {
                if (data_entry.v !== undefined) {
                    return {
                        x: data_entry.x * x_scaling,
                        y: data_entry.y * y_scaling,
                        v: data_entry.v
                    }
                }

                return {
                    x: data_entry.x * x_scaling,
                    y: data_entry.y * y_scaling,
                    r: data_entry.r
                }
            })

            return entry;
        });

        single_view_handler.data.labels = single_view_handler.data.labels.map(label => label * x_scaling);

        if(update_axis_max) {
            single_view_handler.options.scales.x.max *= x_scaling;
            single_view_handler.options.scales.y.max *= y_scaling;
            single_view_handler.options.plugins.zoom.limits.x.max *= x_scaling;
            single_view_handler.options.plugins.zoom.limits.y.max *= y_scaling;
        }

        single_view_handler.options.scales.x.title = {
            display: true,
            text: `Abstand (${axis_label})`
        }

        single_view_handler.options.scales.y.title = {
            display: true,
            text: `Abstand (${axis_label})`
        }
    
        single_view_handler.update();
    }
}

function get_window_borders() {
    let window_start_value = Number(window_start.value.replace(",", "."));
    let window_end_value = Number(window_end.value.replace(",", "."));

    let window_request_param_start = window_start_value !== NaN ? get_index_from_time(window_start_value) : 0;
    let window_request_param_end = window_end_value !== NaN ? get_index_from_time(window_end_value) : time.length - 1;
    
    if(window_request_param_end <= window_request_param_start) {
        window_request_param_end = time.length -1;
    }

    window_start.value = time[window_request_param_start].toFixed(4);
    window_end.value = time[window_request_param_end].toFixed(4);

    display_border(Number(window_start.value), Number(window_end.value));

    return [window_request_param_start, window_request_param_end];
}

function display_border(start, end) {
    if(a_scan_handler !== undefined) {
        a_scan_handler.options.plugins.annotation.annotations.line1 = {
            type: 'line',
            xMin: start,
            xMax: start,
            yMin: -35000,
            yMax: 35000,
            borderWidth: 2,
            borderColor: 'rgb(255, 99, 132)'
        };
        
        a_scan_handler.options.plugins.annotation.annotations.line2 = {
            type: 'line',
            xMin: end,
            xMax: end,
            yMin: -35000,
            yMax: 35000,
            borderWidth: 2,
            borderColor: 'rgb(255, 99, 132)'
        };
        
        a_scan_handler.update();
    }
}

function get_index_from_time(time_value) {
    let closest = time.reduce((prev, curr) => {
        return Math.abs(Number(curr) - time_value) < Math.abs(Number(prev) - time_value) ? curr : prev;
    });

    return time.indexOf(closest);
}

function reset_display() {
    display_mode.value = "";
}

function load_d_scan(channel, start, end, new_mode) {
    const normalized = a_scan_rel.checked ? 0 : 1;

    fetch(`/d_scan?c=${channel}&start=${start}&end=${end}&as_decibel=${normalized}`).then(resp => resp.json())
    .then(d_scan_array => {
        d_scan_array = d_scan_array.map(row => row.map(value => {
            let start_time = Number(time[0]);
            let time_step = Number(time[1]) - start_time;
            return value * time_step;
        }));
        plot_2d_data(d_scan_array, "D-Bild", new_mode);
    });
}

function load_c_scan(channel, start, end, new_mode) {
    const normalized = a_scan_rel.checked ? 0 : 1;

    fetch(`/c_scan?c=${channel}&start=${start}&end=${end}&as_decibel=${normalized}`).then(resp => resp.json())
    .then(c_scan_array => {
        plot_2d_data(c_scan_array, "C-Bild", new_mode);
    });
}

function plot_2d_data(scan_array, title_text, new_mode) {
    const canvas = document.getElementById("single_view_visualize");
    const matrix_format = prepare_array(scan_array);
    const array_max = Math.max.apply(Math, matrix_format.slice(0, -1).map((o) => {return Math.max.apply(Math, o.data.map(element => {return element.v}))}));
    const array_min = Math.min.apply(Math, matrix_format.slice(0, -1).map((o) => {return Math.min.apply(Math, o.data.map(element => {return element.v}))}));

    last_entry = matrix_format.length - 1;
    
    if(single_view_handler !== undefined) {
        single_view_handler.data.datasets = matrix_format;
        
        single_view_handler.options.plugins.title.text = title_text;

        if(a_scan_scale_x !== 1) {
            update_axis_scaling(global_header.res_x, global_header.res_y, false, 'mm');
        }
        else {
            single_view_handler.update();
        }
    }
    else {
        const chart_config = {
            data: {
                datasets: matrix_format
            },
            options: {
                devicePixelRatio: 4,
                onClick: (event) => {
                    const canvas_pos = Chart.helpers.getRelativePosition(event, single_view_handler);
                    a_scan_x = Math.min(Math.max(Math.round(single_view_handler.scales.x.getValueForPixel(canvas_pos.x) / a_scan_scale_x), 0), global_header.samples_x - 1);
                    a_scan_y = Math.min(Math.max(Math.round(single_view_handler.scales.y.getValueForPixel(canvas_pos.y) / a_scan_scale_y), 0), global_header.samples_y - 1) - 1;
                    displayAScan(channel, a_scan_x, a_scan_y, false);
                    single_view_handler.data.datasets[last_entry].data[0].x = a_scan_x * a_scan_scale_x;
                    single_view_handler.data.datasets[last_entry].data[0].y = a_scan_y * a_scan_scale_y + 0.5 * a_scan_scale_y;
                    single_view_handler.update();
                },
                aspectRatio: 1,
                scales: {
                    x: {
                        type: 'linear',
                        min: 0.0,
                        max: Math.max(scan_array[0].length, scan_array.length),
                        title: {
                            display: true,
                            text: `Abstand (${points_scaling.checked ? "Punkte" : "mm"})`
                        },
                        stacked: true
                    },
                    y: {
                        min: 0.0,
                        type: 'linear',
                        max: Math.max(scan_array.length, scan_array[0].length),
                        title: {
                            display: true,
                            text: `Abstand (${points_scaling.checked ? "Punkte" : "mm"})`
                        },
                        stacked: true
                    }
                },
                plugins: {
                    zoom: {
                        limits: {
                            x: {
                                min: 0.0,
                                max: Math.max(scan_array[0].length, scan_array.length)
                            },
                            y: {
                                min: 0.0,
                                max: Math.max(scan_array[0].length, scan_array.length)
                            }
                        },
                        zoom: {
                            wheel: {
                                enabled: true
                            },
                            pinch: {
                                enabled: true
                            },
                            mode: 'xy',
                            drag: {
                                enabled: true,
                                modifierKey: 'alt'
                            }
                        }
                    },
                    tooltip: {
                        callbacks: {
                            label: function label(context) {
                                if (context.raw.v === undefined) {
                                    return `A-Bild (${context.raw.x}, ${context.raw.y})`;
                                }

                                return context.raw.v !== Math.trunc(context.raw.v) ? context.raw.v.toFixed(4) : context.raw.v;
                            }
                        }
                    },
                    title: {
                        display: true,
                        text: title_text
                    },
                    legend: {
                        display: false
                    }
                }
            }
        };

        single_view_handler = new Chart(canvas, chart_config);

        if(a_scan_scale_x !== 1) {
            update_axis_scaling(global_header.res_x, global_header.res_y, true, points_scaling.checked ? "Punkte" : "mm");
        }
    }

    if(new_mode) {
        data_min = array_min;
        data_max = array_max;

        if(data_max !== Math.floor(data_max)) {
            color_bar_max.textContent = data_max.toFixed(4);
            color_bar_min.textContent = data_min.toFixed(4);
        }
        else {
            color_bar_max.textContent = data_max.toFixed(0);
            color_bar_min.textContent = data_min.toFixed(0);
        }

        color_range_min.value = color_bar_min.textContent;
        color_range_max.value = color_bar_max.textContent;

        const event = new Event('change');
        color_range_max.dispatchEvent(event);
    }
}

function prepare_array(scan) {
    let datasets = [];

    let current_row = -1;
    let current_sample;

    for(let i = 0; i < scan.length * scan[0].length; i++) {
        const row = Math.trunc(i / scan[0].length);
        const col = Math.trunc(i % scan[0].length);

        const v = scan[row][col];

        if (current_row < row) {
            current_row++;

            if(current_sample !== undefined) {
                datasets.push(current_sample);
            }

            current_sample = {
                type: 'bar',
                data: [],
                barPercentage: 1,
                categoryPercentage: 1,
                order: 2
            }
        }

        current_sample.data.push({
            x: col,
            y: 1,
            v: v
        })
    }

    let array_max;
    let array_min;

    if(color_range_min.value === "") {
        array_max = Math.max.apply(Math, datasets.map((o) => {return Math.max.apply(Math, o.data.map(element => {return element.v}))}));
        array_min = Math.min.apply(Math, datasets.map((o) => {return Math.min.apply(Math, o.data.map(element => {return element.v}))}));
    }
    else {
        array_max = Number(color_range_max.value);
        array_min = Number(color_range_min.value);
    }

    datasets = datasets.map(entry => {
        let colors = [];

        entry.data.forEach((element) => {
            const value = (element.v - array_min) / (array_max - array_min);
            colors.push(color_mapping(value));
        });

        entry.backgroundColor = colors;

        return entry;
    });

    datasets.push({
        type: 'bubble',
        label: 'Ausgewähltes A-Bild',
        data: [{
            x: a_scan_x,
            y: a_scan_y + 0.5 * a_scan_scale_y,
            r: 5
        }],
        backgroundColor: 'rgb(255, 255, 255)',
        borderWidth: 1,
        borderColor: 'rgb(0, 0, 0)',
        order: 1
    });

    return datasets;
}
