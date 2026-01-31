function updateWeatherReport(rep){
    document.getElementById("weather-report").innerHTML = rep;
}
const proto = location.protocol.startsWith("https") ? "wss" : "ws"
const wsUri = `${proto}://${location.host}/ws`
const socket = new WebSocket(wsUri)

socket.onmessage = ev => {
    console.log('Received: ' + ev.data, 'message')
    updateWeatherReport(ev.data)
}
document.getElementById("weather-report").innerHTML="";