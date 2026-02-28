"use strict";

function createLayers(){
    let layers = new ol.Collection();
    let layers_group = new ol.layer.Group({
        layers: layers,
    });
    layers.push(new ol.layer.Vector({
        name: 'tfrs',
        title: 'TFRs',
        type: 'overlay',
        opacity: 1,
        visible: true,
        zIndex: 99,
        source: new ol.source.Vector({
            url: 'https://raw.githubusercontent.com/wiedehopf/tar1090-aux/refs/heads/master/tfrs.geojson',
            format: new ol.format.GeoJSON(),
            attributions: 'TFRs courtesy of <a href="https://github.com/wiedehopf/tar1090-aux" target="_blank">tar1090-aux</a>.'
        }),
        style: new ol.style.Style({
            fill: new ol.style.Fill({
                color : [255, 0, 0, 0.2]
            }),
            stroke: new ol.style.Stroke({
                color: [255, 0, 0, 0.9],
                width: 1
            }),
        }),

    }));
    layers.push(new ol.layer.Vector({
        name: 'firs',
        title: 'FIRs',
        type: 'overlay',
        opacity: 1,
        visible: false,
        zIndex: 3,
        source: new ol.source.Vector({
            url: "https://gist.githubusercontent.com/LC43/5d6a009d83172d308a01a1c864b71e68/raw/cb57c489cb4dd60c0143ca41e682df0269cead14/fir.geojson",
            format: new ol.format.GeoJSON(),
            attributions: 'FIR boundaries from <a href="https://gist.github.com/LC43/5d6a009d83172d308a01a1c864b71e68">LC43</a> on github.'
        }),
        style: new ol.style.Style({
            stroke: new ol.style.Stroke({
                color: [0, 74, 193, 0.9],
                width: 3
            }),
        }),
    }));
    layers.push(new ol.layer.Vector({
        type: 'overlay',
        title: 'US A2A Refueling',
        name: 'us-a2a',
        zIndex: 2,
        visible: false,
        source: new ol.source.Vector({
            url: '/geojson/US_A2A_refueling.geojson',
            transition: 0,
            format: new ol.format.GeoJSON()
        }),
        style:  new ol.style.Style({
            fill: new ol.style.Fill({
                color : [52, 50, 168, 0.3]
            }),
            stroke: new ol.style.Stroke({
                color: [52, 50, 168, .9],
                width: 1
            }),
        }),
    }));
    layers.push(new ol.layer.Vector({
        type: 'overlay',
        title: 'Special Use Airspace',
        name: 'sua',
        zIndex: 10,
        visible: false,
        source: new ol.source.Vector({
            url: 'https://opendata.arcgis.com/datasets/dd0d1b726e504137ab3c41b21835d05b_0.geojson',
            transition: 0,
            format: new ol.format.GeoJSON({
                defaultDataProjection: 'EPSG:4326',
                projection: 'EPSG:3857'
            })
        }),
        style: function style(feature) {
            let type = feature.getProperties().TYPE_CODE;
            if (type == "P" || type == "R" || type == "W") {
                return new ol.style.Style({
                    stroke: new ol.style.Stroke({
                        color: 'rgba(72, 149, 239, 1)',
                        width: 2
                    }),
                    fill: new ol.style.Fill({
                        color: 'rgba(72, 149, 239, 0.3)',
                    })
                })
            } else if (type == "A" || type == "MOA") {
                return new ol.style.Style({
                    stroke: new ol.style.Stroke({
                        color: 'rgba(133, 45, 69, 1)',
                        width: 2
                    }),
                    fill: new ol.style.Fill({
                        color : 'rgba(133, 45, 69, 0.3)'
                    })
                });
            }
        }
    }));
    let adsb_menu = document.getElementById('adsb-menu');
    layers.forEach(layer => {
        const box = document.createElement('input');
        box.type = 'checkbox';
        box.id = layer.get('name');
        box.name = layer.get('name');
        box.checked = layer.getVisible();
        const label = document.createElement('label');
        label.className = 'layer-label';
        label.htmlFor = layer.get('name');
        label.textContent = layer.get('title');
        adsb_menu.appendChild(box);
        adsb_menu.appendChild(label);
        adsb_menu.appendChild(document.createElement('br'));
        box.addEventListener('change',function(){layer.setVisible(box.checked)});
    })
    return layers_group;
}

var map = new ol.Map({
    layers: [
        new ol.layer.Tile({
            source: new ol.source.OSM(),
        }),
        createLayers(),
    ],
    target: 'map',
    view: new ol.View({
        center: ol.proj.fromLonLat([-73.9821789,40.694359]),
        zoom: 11
    })
});
const vectorSource = new ol.source.Vector();

async function loadStations(){
    const response = await fetch('stations.json');
    const stations = await response.json();
    console.log("stations", stations);
    stations.forEach(station => {
        console.log(station.callsign);
        console.log(station.symbol)
        const iconStyle =getSymbol(station.symbol);
        const feature = new ol.Feature({
            geometry: new ol.geom.Point(ol.proj.fromLonLat([station.lon,station.lat])),
            callsign: station.callsign,
            comment: station.packet,
            timestamp: station.time,
        });
        feature.setStyle(iconStyle)
        vectorSource.addFeature(feature);
    })
    const vectorLayer = new ol.layer.Vector({
        visible: true,
        source: vectorSource,
    });
    map.addLayer(vectorLayer);
}

const aircraftVectors= new ol.source.Vector();

async function loadAircraftVectors(){
    const response = await fetch("aircraft.json")
    const json_aircraft = await response.json();
    console.log(json_aircraft);
    const aircrafts = await json_aircraft.aircraft;
    console.log("aircraft", aircrafts);
    aircrafts.forEach(aircraft => {
        console.log(aircraft);
        const point = new ol.geom.Point(ol.proj.fromLonLat([station.lon,station.lat]));
        const feature = new ol.Feature({
            geometry: point,
        });
        aircraftVectors.addFeature(feature);
    })
    const wxStyle = new ol.style.Style({
        image: new ol.style.Circle({
            radius: 7,
            stroke: new ol.style.Stroke({ color: 'rgba(0, 200, 128, 1)', width: 1 })
        })
    });

    const aircraftLayer = new ol.layer.Vector({
        visible: true,
        source: vectorSource,
        style: wxStyle
    });
    map.addLayer(aircraftLayer);
}

loadAircraftVectors()
loadStations();
