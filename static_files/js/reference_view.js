const file_selector_1 = document.getElementById('file_1');
const file_selector_2 = document.getElementById('file_2');

const charts = {}

const chart_data = {}

function load_scan(body_content, canvas_id, title) {
    fetch('/import', { method: 'POST', body: body_content }).then(resp => {
        if (resp.ok) {
            resp.json().then(content => {
                create_plot(content, canvas_id, title);

                plot_reference();
            });
        }
        else {
            alert('Data import failed!');
        }
    })
}

function create_plot(content, canvas_id, title) {
    const canvas = document.getElementById(canvas_id);
    const data = prepare_array(content, canvas_id);

    if (charts.hasOwnProperty(canvas_id) && charts[canvas_id] !== undefined) {
        charts[canvas_id].destroy();
        charts[canvas_id] = undefined;
    }

    if (canvas_id !== 'reference') {
        chart_data[canvas_id] = content;   
    }

    charts[canvas_id] = new Chart(canvas, {
        data: {
            datasets: data
        },
        options: {
            devicePixelRatio: 4,
            aspectRatio: 1,
            scales: {
                x: {
                    type: 'linear',
                    min: 0.0,
                    max: Math.max(content[0].length, content.length),
                    title: {
                        display: true,
                        text: 'Abstand (Punkte)'
                    },
                    stacked: true
                },
                y: {
                    min: 0.0,
                    type: 'linear',
                    max: Math.max(content.length, content[0].length),
                    title: {
                        display: true,
                        text: 'Abstand (Punkte)'
                    },
                    stacked: true
                }
            },
            plugins: {
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
                    text: title
                },
                legend: {
                    display: false
                }
            }
        }
    });
}

function load_data(file_content, canvas_id, title) {
    const reader = new FileReader();
    reader.readAsArrayBuffer(file_content);

    reader.onloadend = () => {
        load_scan(reader.result, canvas_id, title);
    };
}

file_selector_1.addEventListener('change', (event) => {
    load_data(event.target.files[0], 'scan_1', 'C-Bild 1');
});

file_selector_2.addEventListener('change', (event) => {
    load_data(event.target.files[0], 'scan_2', 'C-Bild 2');
});

function plot_reference() {
    const keys = Object.keys(chart_data);

    if (keys.length === 2) {
        let data1 = chart_data[keys[0]];
        let data2 = chart_data[keys[1]];

        while (data1.length !== data2.length) {
            if (data1.length < data2.length) {
                const new_array = new Array(data1[0].length).fill(0);
                data1.push(new_array);
            }
            else {
                const new_array = new Array(data2[0].length).fill(0);
                data2.push(new_array);
            }
        }

        while (data1[0].length !== data2[0].length) {
            if (data1[0].length < data2[0].length) {
                data1.forEach((element) => {
                    element.unshift(0);
                });
            } else {
                data2.forEach((element) => {
                    element.unshift(0);
                });
            }
        }

        const reference = [];

        for (let i = 0; i < data1.length; i++) {
            const row = [];

            for (let j = 0; j < data1[0].length; j++) {
                row.push(data1[i][j] - data2[i][j]);
            }

            reference.push(row);
        }

        create_plot(reference, 'reference', 'Referenzbild');
    }
}

function prepare_array(scan, canvas_id) {
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

    let array_max = Math.max.apply(Math, datasets.map((o) => {return Math.max.apply(Math, o.data.map(element => {return element.v}))}));
    let array_min = Math.min.apply(Math, datasets.map((o) => {return Math.min.apply(Math, o.data.map(element => {return element.v}))}));

    datasets = datasets.map(entry => {
        let colors = [];

        entry.data.forEach((element) => {
            const value = (element.v - array_min) / (array_max - array_min);
            if (canvas_id !== 'reference') {
                colors.push(color_fz_u(value));   
            }
            else {
                colors.push(red_white_blue(1 / (1 + Math.exp(-0.1 * Math.log2(2 + Math.sqrt(3)) * element.v))));
            }
        });

        entry.backgroundColor = colors;

        return entry;
    });

    return datasets;
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
