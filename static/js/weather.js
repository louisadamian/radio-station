const weatherText = document.getElementsByClassName("weather-report")
weatherText.innerHTML = "Weather: 60&deg;F partly cloudy &#9925;";
/** @type {WebSocket | null} */
let socket = null;

function updateWeatherReport(report){
    document.getElementById("weather-report").innerHTML = "Weather: " + report;
}

function disconnect() {
    if (socket) {
        console.log('Disconnecting...')
        socket.close()
        socket = null
    }
}

function connect(){
    disconnect();
    const { location } = window;
    const proto = location.protocol.startsWith("https")? "wss" : "ws";
    const wsUri = `${proto}://${location.host}/ws`;
    socket = new WebSocket(wsUri);

    socket.onopen= () => {
        console.log('Connected');
    }
    socket.onmessage = ev => {
        updateWeatherReport(ev.data)
        console.log('Received: ' + ev.data, 'message')
    }
    socket.onclose = () => {
        console.log('Disconnected')
        socket = null
    }
}
connect()