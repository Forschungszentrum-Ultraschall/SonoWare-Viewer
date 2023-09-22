const display_mode = document.getElementById('display_mode');
const window_start = document.getElementById('aperture_start');
const window_end = document.getElementById('aperture_end');
const points_scaling = document.getElementById('points');
const mm_scaling = document.getElementById('mm');

let scale = points_scaling.value;
let a_scan_scale_x = 1;
let a_scan_scale_y = 1;

points_scaling.checked = true;
display_mode.value = "";

display_mode.addEventListener('change',(_) => {
    update_borders();
});

window_start.addEventListener('change', (_) => {
    update_borders();
});

window_end.addEventListener('change', (_) => {
    update_borders();
});

points_scaling.addEventListener('click', (event) => {
    if(scale !== event.target.value) {
        scale = event.target.value;
        a_scan_scale_x = 1;
        a_scan_scale_y = 1;
        update_axis_scaling(1 / global_header.res_x, 1 / global_header.res_y);
    }
});

mm_scaling.addEventListener('click', (event) => {
    if(scale !== event.target.value) {
        scale = event.target.value;
        a_scan_scale_x = global_header.res_x;
        a_scan_scale_y = global_header.res_y;
        update_axis_scaling(global_header.res_x, global_header.res_y);
    }
});

function update_borders() {
    let borders = get_window_borders();

    switch(display_mode.value) {
        case 'c-scan':
            load_c_scan(channel, borders[0], borders[1]);
            break;
        case 'd-scan':
            load_d_scan(channel, borders[0], borders[1]);
            break;
    }
}

function update_axis_scaling(x_scaling, y_scaling, update_axis_max = true) {
    if (single_view_handler !== undefined) {
        let display_dataset = single_view_handler.data.datasets[0].data;
        display_dataset = display_dataset.map(value => {
            return {
                x: value.x * x_scaling,
                y: value.y * y_scaling,
                v: value.v
            };
        });

        single_view_handler.data.datasets[1].data = [
            {
                x: a_scan_x * a_scan_scale_x,
                y: a_scan_y * a_scan_scale_y,
                r: 5
            }
        ]

        single_view_handler.data.datasets[0].data = display_dataset;

        if(update_axis_max) {
            single_view_handler.options.scales.x.max *= x_scaling;
            single_view_handler.options.scales.y.max *= y_scaling;
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

    window_start.value = time[window_request_param_start];
    window_end.value = time[window_request_param_end];

    display_border(Number(window_start.value), Number(window_end.value));

    return [window_request_param_start, window_request_param_end];
}

function display_border(start, end) {
    if(a_scan_handler !== undefined) {
        a_scan_handler.options.plugins.annotation = {
            annotations: {
                line1: {
                    type: 'line',
                    xMin: start,
                    xMax: start,
                    yMin: -35000,
                    yMax: 35000,
                    borderWidth: 2,
                    borderColor: 'rgb(255, 99, 132)'
                },
                line2: {
                    type: 'line',
                    xMin: end,
                    xMax: end,
                    yMin: -35000,
                    yMax: 35000,
                    borderWidth: 2,
                    borderColor: 'rgb(255, 99, 132)'
                }
            }
        }
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

function load_d_scan(channel, start, end) {
    fetch(`/d_scan/${channel}/${start}/${end}`).then(resp => resp.json())
    .then(d_scan_array => {
        d_scan_array = d_scan_array.map(row => row.map(value => {
            let start_time = Number(time[0]);
            let time_step = Number(time[1]) - start_time;
            return value * time_step;
        }));
        plot_2d_data(d_scan_array, "D-Bild");
    });
}

function load_c_scan(channel, start, end) {
    fetch(`/c_scan/${channel}/${start}/${end}`).then(resp => resp.json())
    .then(c_scan_array => {
        plot_2d_data(c_scan_array, "C-Bild");
    });
}

function plot_2d_data(scan_array, title_text) {
    const canvas = document.getElementById("single_view_visualize");
    const matrix_format = prepare_array(scan_array);
    const array_max = Math.max.apply(Math, matrix_format.map((o) => {return o.v}));
    const array_min = Math.min.apply(Math, matrix_format.map((o) => {return o.v}));
    
    if(single_view_handler !== undefined) {
        single_view_handler.data.datasets[0].data = matrix_format;
        single_view_handler.data.datasets[0].backgroundColor = ({raw}) => {
            const value = (raw.v - array_min) / (array_max - array_min);
            return `hsl(${240 * (1 - value)},100%,${75 - Math.abs(25 - 50 * value)}%)`;
        };
        single_view_handler.options.plugins.title.text = title_text;

        if(a_scan_scale_x !== 1) {
            update_axis_scaling(global_header.res_x, global_header.res_y, false);
        }
        else {
            single_view_handler.update();
        }
    }
    else {
        single_view_handler = new Chart(canvas, {
            data: {
                datasets: [
                    {
                        type: 'matrix',
                        data: matrix_format,
                        backgroundColor({raw}) {
                            const value = (raw.v - array_min) / (array_max - array_min);
                            return `hsl(${240 * (1 - value)},100%,${75 - Math.abs(25 - 50 * value)}%)`;
                        },
                        order: 2
                    },
                    {
                        type: 'bubble',
                        label: 'Ausgewähltes A-Bild',
                        data: [{
                            x: a_scan_x,
                            y: a_scan_y,
                            r: 5
                        }],
                        backgroundColor: 'rgba(0, 0, 0, 1)',
                        order: 1
                    }
                ]
            },
            options: {
                onClick: (event) => {
                    const canvas_pos = Chart.helpers.getRelativePosition(event, single_view_handler);
                    a_scan_x = Math.min(Math.max(Math.round(single_view_handler.scales.x.getValueForPixel(canvas_pos.x) / a_scan_scale_x), 0), global_header.samples_x - 1);
                    a_scan_y = Math.min(Math.max(Math.round(single_view_handler.scales.y.getValueForPixel(canvas_pos.y) / a_scan_scale_y), 0), global_header.samples_y - 1);
                    displayAScan(channel, a_scan_x, a_scan_y, false);
                    single_view_handler.data.datasets[1].data[0].x = a_scan_x * a_scan_scale_x;
                    single_view_handler.data.datasets[1].data[0].y = a_scan_y * a_scan_scale_y;
                    single_view_handler.update();
                },
                aspectRatio: scan_array[0].length / scan_array.length,
                scales: {
                    x: {
                        min: 0.0,
                        max: scan_array[0].length
                    },
                    y: {
                        min: 0.0,
                        max: scan_array.length
                    }
                },
                plugins: {
                    tooltip: {
                        callbacks: {
                            label: function label(context) {
                                return context.raw.v;
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
        });
    }
}

function prepare_array(scan) {
    const data = [];

    for(let i = 0; i < scan.length * scan[0].length; i++) {
        const row = Math.trunc(i / scan[0].length);
        const col = Math.trunc(i % scan[0].length);

        const v = scan[row][col];

        data.push({
            x: col,
            y: row,
            v: v
        });
    }

    return data;
}