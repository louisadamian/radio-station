import os.path
from datetime import datetime
from metar import Metar
now = datetime.now()
def metartime(timestr:str):
    day = timestr[0:2]
    hour = timestr[2:4]
    minute = timestr[4:6]
    tz = timestr[6:7]
    isoformat = f"{now.year}-{str(now.month).zfill(2)}-{day}T{hour}:{minute}:00{tz}"
    # print(isoformat)
    return datetime.fromisoformat(isoformat)

def __temp_to_c(temp)->float:
    if temp._units == "C":
        return temp.value()
    if temp._units == "F":
        return (temp.value()- 32) * 5 / 9
    if temp._units == "K":
        return temp.value()-273.15
def __temp_to_f(temp)->float:
    if temp._units == "F":
        return temp.value()
    if temp._units == "C":
        return temp.value()*9/5 + 32
    if temp._units == "K":
        return (temp.value()-273.15)*9/5+32
def __speed_to_kts(speed)->float:
    if(speed._units == "KT"):
        return speed._value
    if(speed._units == "MPS"):
        return speed._value*1.943844
    if(speed._units == "KMH"):
        return speed._value*0.539957
    if(speed._units == "MPH"):
        return speed._value*0.868976
def __speed_to_mph(speed)->float:
    if(speed._units == "MPH"):
        return speed._value
    if(speed._units == "KMH"):
        return speed._value*0.621371
    if(speed._units == "MPS"):
        return speed._value*2.236936
    if(speed._units == "KT"):
        return speed._value*1.150779
def __speed_to_kph(speed)->float:
    if(speed._units == "KMH"):
        return speed._value
    if(speed._units == "MPH"):
        return speed._value*0.621371
    if(speed._units == "KT"):
        return speed._value*1.150779
    if(speed._units == "MPS"):
        return speed._value*2.236936
def __pressure_to_mb(pressure)->float:
    if pressure._units == "MB" or pressure._units == "HPA":
        return pressure._value
    if pressure._units == "IN":
        return pressure._value*33.8639

def __pressure_to_inhg(pressure)->float:
    if pressure._units == "MB" or pressure._units == "HPA":
        return pressure._value/33.8639
    if pressure._units == "IN":
        return pressure._value

def get_weather(path, icao, city_name, units:str="metric"):
    global rep
    dir  = os.path.normpath(path)
    files = os.listdir(dir)
    metars = []
    weather = []
    for f in files:
        if f.endswith(".txt") or f.endswith(".TXT"):
            with open(os.path.join(dir, f)) as file:
                txt = file.read()
                if icao in txt:
                    txt = txt.split("METAR")[1]
                    metars_ = txt.split("=")
                    city = filter(lambda x: x.lstrip().startswith(icao), metars_)
                    metars.append(list(city)[0].lstrip())
                if city_name in txt:
                    weather.append(f)
    times = []
    for _metar in metars:
        timestr = _metar.split()[1]
        times.append(metartime(timestr))
    latest = times.index(max(times))
    obs = Metar.Metar(metars[latest])
    if units == "metric":
        # metar
        temp = __temp_to_c(obs.temp)
        t_unit = "C"
        wind_speed = __speed_to_kph(obs.wind_speed)
        speed_unit="k/h"
        pressure = __pressure_to_mb(obs.press)
        pres_unit="mb"
    if units == "imperial":
        temp = __temp_to_c(obs.temp)
        t_unit = "F"
        wind_speed = __speed_to_mph(obs.wind_speed)
        speed_unit="mph"
        pressure = __pressure_to_inhg(obs.press)
        pres_unit="inHg"
    if units == "nautical":
        temp = __temp_to_c(obs.temp)
        t_unit = "C"
        wind_speed = __speed_to_kts(obs.wind_speed)
        speed_unit="kts"
        pressure = __pressure_to_mb(obs.press)
        pres_unit="mb"
    if units == "aviation":
        temp = __temp_to_c(obs.temp)
        t_unit = "C"
        wind_speed = __speed_to_kts(obs.wind_speed)
        speed_unit="kts"
        pressure = __pressure_to_inhg(obs.press)
        pres_unit="inHg"
    # condition = str(obs.condition).lower()
    print(obs.sky_conditions())
    rep=obs.sky_conditions()
    # if "clr" in condition:
    #     rep = "Clear"
    # else:
    #     if "few" in condition:
    #         rep = "partly cloudy"
    #     if "OVC" in condition:
    #         rep = "overcast"
    #     if "FG" in condition:
    #         rep = "fog"
    #     if "-SN" in condition:
    #         rep += "light snow"
    #     if ""


    return f"weather for {icao} at {times[latest].strftime("%Y-%m-%dT%H:%M")}Z\n{round(temp)}º{t_unit} {rep} wind {round(wind_speed)}{speed_unit} pressure {round(pressure,2)}{pres_unit}"
if __name__ == '__main__':
    path = "/Users/louisadamian/Downloads/txthrs01(2)"
    dir = os.path.normpath("/Users/louisadamian/Downloads/txthrs01(2)")
    files = os.listdir(dir)
    icao = "KBOS"
    city_name = "Boston"
    print(get_weather(path, icao, city_name))