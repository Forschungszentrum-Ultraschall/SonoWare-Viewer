const fileSelector = document.getElementById('file');
const channel_selector = document.getElementById('channel_selector');
const export_button = document.getElementById('export_button');
const a_scan_canvas = document.getElementById('a_scan_view');

let a_select_x_start = 0;
let a_select_x_end = 0;

let content_binary = undefined;
let binary_file_name = '';

// close the application when Tab is closed
window.addEventListener('beforeunload', (_) => {
    fetch("/exit").catch((_) => {});
});

// log the start position, if the user clicks into the A-Scan plot
a_scan_canvas.addEventListener('mousedown', (event) => {
    a_select_x_start = a_scan_handler.scales.x.getValueForPixel(event.offsetX);
});

// update the aperture if user releases the mouse button and pressed the Crtl key
a_scan_canvas.addEventListener('mouseup', (event) => {
    a_select_x_end = a_scan_handler.scales.x.getValueForPixel(event.offsetX);
    
    if(a_select_x_start !== a_select_x_end && event.ctrlKey) {
        let start = a_select_x_start < a_select_x_end ? a_select_x_start : a_select_x_end;
        let end = a_select_x_start > a_select_x_end ? a_select_x_start : a_select_x_end;

        window_start.value = start.toFixed(4);
        window_end.value = end.toFixed(4);
        update_borders();
    }
});

// trigger the data export if user clicks the export button
export_button.addEventListener('click', (_) => {
    let borders = get_window_borders();

    const output_name = `Messdaten ${binary_file_name.replace('.sdt', '')}`;
    const requested_name = prompt("Unter welchem Dateinamen sollen die Daten exportiert werden?", output_name);

    fetch(`/export?channel=${channel_selector.value.split(' ')[1] - 1}&start=${borders[0]}&end=${borders[1]}&name=${requested_name}`,
        { method: 'POST' })
        .then(resp => {
            resp.text().then(text => {
                alert(text);
            });
        });
});

// read and show data if the user selects a file
fileSelector.addEventListener('change', (event) => {
    const reader = new FileReader();
    reader.readAsArrayBuffer(event.target.files[0]);

    reader.onloadend = () => {
        content_binary = reader.result;
        binary_file_name = event.target.files[0].name;
        load_binary_data(undefined);
    };
});

function load_binary_data() {
    if (content_binary === undefined) {
        return;
    }

    fetch('/data/sonoware', {
        method: 'POST',
        headers: {
            "Content-Type": "application/octet-stream"
        },
        body: content_binary
    }).then(response => {
        if(response.ok) {
            const footer = document.getElementById('data_info');

            fetch("/header").then(resp => {
                if(resp.ok) {
                    resp.json().then(header => {
                        global_header = header;
                        reset_views();
                        reset_display();
                        initializeAScan(header);
    
                        footer.innerText = `${binary_file_name} - ${header.format} Version ${header.version}`;
                    });
                }
                else {
                    resp.text().then(text => {
                        alert(text);
                    })
                }
            });
        }
        else {
            response.text().then(text => {
                alert(text);
            });
        }
    });
}

/**
 * Update the settings for the A-Scan plot
 * @param {object} header loaded data header
 */
function initializeAScan(header) {
    reloadChannels(header.channels);
    a_scan_x = Math.trunc(header.samples_x / 2);
    a_scan_y = Math.trunc(header.samples_y / 2);
    displayAScan(channel, a_scan_x, a_scan_y, true);
}

/**
 * Destroy all loaded plots
 */
function reset_views() {
    if (a_scan_handler !== undefined) {
        a_scan_handler.destroy();
        a_scan_handler = undefined;
    }

    if (single_view_handler !== undefined) {
        single_view_handler.destroy();
        single_view_handler = undefined;
    }

    if (multi_view_handler_left !== undefined) {
        multi_view_handler_left.destroy();
        multi_view_handler_left = undefined;
    }

    if (multi_view_handler_right !== undefined) {
        multi_view_handler_right.destroy();
        multi_view_handler_right = undefined;
    }
}

/**
 * Load the specified A-Scan and display it
 * @param {Number} c Channel index
 * @param {Number} x Column index
 * @param {Number} y Row index
 * @param {boolean} new_data A new plot will be created
 */
function displayAScan(c, x, y, new_data) {
    fetch(`/a_scan?c=${c}&x=${x}&y=${y}`).then(resp => {
        if(resp.ok) {
            resp.json().then(a_scan_data => {
                plot_a_scan(a_scan_data.scan, a_scan_data.filtered_scan, a_scan_data.time_start, a_scan_data.time_step, new_data);
            });
        }
        else {
            resp.text().then(text => {
                alert(text);
            });
        }
    })
}

/**
 * Create settings for the A-Scan plot and draw it
 * @param {Array<Number>} samples Measured A-Scan
 * @param {Array<Number>} filtered_samples Filtered A-Scan
 * @param {Number} time_start First time value
 * @param {Number} time_step Time resolution
 * @param {boolean} new_data New plot will be created
 */
function plot_a_scan(samples, filtered_samples, time_start, time_step, new_data) {
    const a_scan_canvas = document.getElementById("a_scan_view");

    time = [...Array(samples.length).keys()];
    time = time.map((value) => (value * time_step / 1000 + time_start));

    let time_end = time.slice(-1);

    let y_label = 'Amplitude (%)';

    if(new_data === true) {
        window_start.value = time[0];
        window_end.value = time_end;

        if(display_mode !== undefined && display_mode.value == '') {
            display_mode.value = 'c-scan';
            const event = new Event('change');
            display_mode.dispatchEvent(event);
        }
    }

    if (a_scan_handler !== undefined) {
        a_scan_handler.data.datasets[0].data = samples;
        a_scan_handler.data.datasets[1].data = filtered_samples;
        a_scan_handler.data.datasets[0].labels = time;
        a_scan_handler.data.datasets[1].labels = time;
        a_scan_handler.options.scales.x.max = time_end;
        a_scan_handler.options.plugins.zoom.limits.x.max = time_end;
        a_scan_handler.update();
    }
    else {
        a_scan_handler = new Chart(a_scan_canvas, {
            type: 'line',
            data: {
                labels: time,
                datasets: [{
                    label: "A-Bild",
                    data: samples,
                    fill: false,
                    pointRadius: 0,
                    borderColor: 'rgba(11, 59, 106, 0.5)',
                    order: 0
                }, {
                    label: "Gefiltertes A-Bild",
                    data: filtered_samples,
                    fill: false,
                    pointRadius: 0,
                    borderColor: 'rgb(11, 59, 106)',
                    order: 1
                }, {
                    label: "",
                    data: Array(time.length).fill(0),
                    fill: false,
                    pointRadius: 0,
                    borderColor: 'rgb(200, 200, 200)',
                    order: 2
                }]
            },
            options: {
                devicePixelRatio: 4,
                aspectRatio: 1,
                scales: {
                    x: {
                        type: 'linear',
                        title: {
                            display: true,
                            text: "Zeit (ms)"
                        },
                        min: time[0],
                        max: time_end
                    },
                    y: {
                        min: -1,
                        max: 1,
                        title: {
                            display: true,
                            text: y_label
                        }
                    }
                },
                plugins: {
                    title: {
                        display: true,
                        text: "A-Bild"
                    },
                    zoom: {
                        limits: {
                            x: {
                                min: time[0],
                                max: time_end
                            },
                            y: {
                                min: -1,
                                max: 1
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
                    legend: {
                        display: false
                    }
                }
            }
        });
    }
}

/**
 * Create options for all loaded channels
 * @param {Number} channels Number of channels
 */
function reloadChannels(channels) {
    const channel_selector = document.getElementById("channel_selector");

    while(channel_selector.childNodes.length > 1) {
        channel_selector.removeChild(channel_selector.lastChild);
    }

    for(let i = 0; i < channels; i++) {
        const channel_option = document.createElement("option");
        channel_option.text = `Kanal ${i + 1}`;

        channel_selector.add(channel_option);
    }

    channel_selector.value = "Kanal 1";
}