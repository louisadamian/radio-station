const weatherTexts = document.getElementsByClassName("weather-report")
weatherTexts[0].innerHTML = "Weather: 60&deg;F partly cloudy &#9925;";

function updateWeatherReport(){
    rep = "Weather: "+"53&deg;F Coudy ☁️";
    document.getElementsById("weather-report").innerHTML = rep;
}
function onImgSelect(){
    changeGoesImage(document.getElementById("goes-select").value);

}