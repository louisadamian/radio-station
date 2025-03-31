const weatherTexts = document.getElementsByClassName("title-weather")
weatherTexts[0].innerHTML = "Weather: 60&deg;F partly cloudy &#9925;";
base_path ="goes_latest"
ext = ".png"
function changeGoesImage(layer){
    let img = document.getElementById("goesImg");
    switch (layer){
        case "geocolor":
            img.src="https://cdn.star.nesdis.noaa.gov/GOES16/ABI/CONUS/GEOCOLOR/5000x3000.jpg";
            break;
        case "glm-extent3":
            img.src="https://cdn.star.nesdis.noaa.gov/GOES16/GLM/CONUS/EXTENT3/5000x3000.jpg";
            break;
        case "airmass":
            img.src="https://cdn.star.nesdis.noaa.gov/GOES16/ABI/CONUS/AirMass/5000x3000.jpg";
            break;
        case"firetemperature-rgb":
            img.src="https://cdn.star.nesdis.noaa.gov/GOES16/ABI/CONUS/FireTemperature/5000x3000.jpg";

            break;
    }
    // img.src = base_path + "_" + layer + ext;
}
function onImgSelect(){
    changeGoesImage(document.getElementById("goes-select").value);

}