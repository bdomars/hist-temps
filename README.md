# hist-temps
Fetch historical temperature data from FMI

Hihly specialized (read: hardcoded) app to run on our office pi and periodically fetch outside air temperature to
our air quality monitoring solution. Currently expects to fetch all data to a bucket called 'fmi' in Influxdb2.

## Usage
```
    hist-temps [OPTIONS] --place <PLACE> --starttime <STARTTIME> --endtime <ENDTIME>

OPTIONS:
    -e, --endtime <ENDTIME>        Timestamp in ISO 8601
    -h, --help                     Print help information
    -p, --place <PLACE>            A place name that is passed to the WFS endpoint (eg. a city)
    -s, --starttime <STARTTIME>    Timestamp in ISO 8601
    -V, --version                  Print version information
    -w, --write-influxdb           Write data to influx
```
