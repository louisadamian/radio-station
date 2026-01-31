

function initAdsB(){
    let topnav = document.getElementById("sidenav-topnav");
    const box = document.createElement('input');
    box.type = 'checkbox';
    box.id = layer.get('name');
    box.name = layer.get('name');
    box.checked = layer.getVisible();
    const label = document.createElement('label');
    label.className = 'layer-label';
    label.htmlFor = layer.get('name');
    label.textContent = layer.get('title');
    topnav.appendChild(box);
    topnav.appendChild(label);

}

function init(){

    let layer_button =document.getElementById("layer-button")
    layer_button.classList.toggle('active');

}
init()
