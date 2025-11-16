"use strict";

function toggleLayer() {

}
function createLayers(){
    let layers = new ol.Collection();
    let adsb_menu = document.getElementById('adsb-menu');

    let layers_group = new ol.layer.Group({
        layers: layers,
    });
    layers.push(new ol.layer.Vector({
        source: new ol.source.Vector({
            url: "/geojson/US_A2A_refueling.geojson",
            format: new ol.format.GeoJSON(),
        }),
        style: new ol.style.Style({
            fill: new ol.style.Fill({
                color : [52, 50, 168, 0.3]
            }),
            stroke: new ol.style.Stroke({
                color: [52, 50, 168, .9],
                width: 1
            }),
        }),
        name: 'us-a2a',
        title: 'US A2A refueling',
        type: 'overlay',
        opacity: 1,
        visible: true,
        zIndex: 20,
    }));
    layers.push(new ol.layer.Vector({
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
        name: 'tfrs',
        title: 'TFRs',
        type: 'overlay',
        opacity: 1,
        visible: true,
        zIndex: 99,
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
    layers.forEach(layer => {
        adsb_menu.innerHTML += '<input type="checkbox" id="'+layer.get('name')+'" name="'+layer.get('name')+ '"/> <label class="layer-label">'+layer.get('title')+'</label><br>\n';
        let box = document.getElementById(layer.get('name'))
        box.addEventListener('change',function(){layer.setVisible(box.checked)});
        box.checked = layer.getVisible();

    });
    return layers_group;
}

var map = new ol.Map({
    layers: [
        new ol.layer.Tile({
            source: new ol.source.OSM()
        }),
        createLayers(),
    ],
    target: 'map',
    view: new ol.View({
        center: ol.proj.fromLonLat([-73.9821789,40.694359]),
        zoom: 11
    })
});


