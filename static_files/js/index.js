const fileSelector = document.getElementById('file');
const channel_selector = document.getElementById('channel_selector');
const export_button = document.getElementById('export_button');
const a_scan_canvas = document.getElementById('a_scan_view');

let a_select_x_start = 0;
let a_select_x_end = 0;

a_scan_canvas.addEventListener('mousedown', (event) => {
    a_select_x_start = a_scan_handler.scales.x.getValueForPixel(event.offsetX);
});

a_scan_canvas.addEventListener('mouseup', (event) => {
    a_select_x_end = a_scan_handler.scales.x.getValueForPixel(event.offsetX);
    
    if(a_select_x_start !== a_select_x_end) {
        let start = a_select_x_start < a_select_x_end ? a_select_x_start : a_select_x_end;
        let end = a_select_x_start > a_select_x_end ? a_select_x_start : a_select_x_end;

        window_start.value = start.toFixed(8);
        window_end.value = end.toFixed(8);
        update_borders();
    }
});

export_button.addEventListener('click', (_) => {
    let borders = get_window_borders();
    fetch(`/export?channel=${channel_selector.value.split(' ')[1] - 1}&start=${borders[0]}&end=${borders[1]}`)
        .then(resp => {
            if(resp.ok) {
                resp.blob().then((blob) => {
                    var a = document.createElement('a');
                    document.body.appendChild(a);
                    a.style = 'display: none';

                    var url = window.URL.createObjectURL(blob);
                    a.href = url;
                    a.download = "Messdaten.zip";
                    a.click();
                    window.URL.revokeObjectURL(url);
                });
            }
        })
});

fileSelector.addEventListener('change', (event) => {
    const reader = new FileReader();
    reader.readAsArrayBuffer(event.target.files[0]);

    reader.onloadend = () => {
        const content = reader.result;
        fetch("/data/sonoware", {
            method: "POST",
            headers: {
                "Content-Type": "application/octet-stream"
            },
            body: content
        }).then(response => {
            if(response.ok) {
                const footer = document.getElementById('data_info');
                const file_name = event.target.files[0].name;

                fetch("/header").then(resp => resp.json().then(header => {
                    global_header = header;
                    reset_views();
                    reset_display();
                    initializeAScan(header);

                    footer.innerText = `${file_name} - ${header.format} Version ${header.version}`;
                }));
            }
        });
    };
});

function initializeAScan(header) {
    reloadChannels(header.channels);
    a_scan_x = Math.trunc(header.samples_x / 2);
    a_scan_y = Math.trunc(header.samples_y / 2);
    displayAScan(channel, a_scan_x, a_scan_y, true);
}

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

function displayAScan(c, x, y, new_data) {
    fetch(`/a_scan/${c}/${x}/${y}`).then(resp => resp.json())
    .then(a_scan_data => {
        plot_a_scan(a_scan_data.scan, a_scan_data.time_start, a_scan_data.time_step, new_data);
    });
}

function plot_a_scan(samples, time_start, time_step, new_data) {
    const a_scan_canvas = document.getElementById("a_scan_view");

    time = [...Array(samples.length).keys()];
    time = time.map((value) => (value * time_step / 1000 + time_start).toFixed(8));

    let time_end = time.slice(-1);

    if(new_data === true) {
        window_start.value = time[0];
        window_end.value = time_end;

        if(display_mode !== undefined && display_mode.value == '') {
            display_mode.value = 'c-scan';
            update_borders();
        }
    }

    if (a_scan_handler !== undefined) {
        a_scan_handler.data.datasets[0].data = samples;
        a_scan_handler.data.datasets[0].labels = time;
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
                    borderColor: 'rgb(54, 162, 235)'
                }]
            },
            options: {
                scales: {
                    x: {
                        type: 'linear',
                        title: {
                            display: true,
                            text: "Zeit (ms)"
                        },
                        min: 0,
                        max: time_end
                    },
                    y: {
                        min: -35000,
                        max: 35000
                    }
                },
                maintainAspectRatio: false,
                plugins: {
                    zoom: {
                        limits: {
                            x: {
                                min: 0,
                                max: time_end
                            },
                            y: {
                                min: -35000,
                                max: 35000
                            }
                        },
                        zoom: {
                            wheel: {
                                enabled: true
                            },
                            pinch: {
                                enabled: true
                            },
                            mode: 'xy'
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