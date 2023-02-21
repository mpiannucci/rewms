# rewms

Restyle WMS raster images to optimize them for webgl rendering

## Building and running

### With Cargo

To run against the IOOS EDS WMS:

```
DOWNSTREAM=eds.ioos.us cargo run
```

### With Docker

First build the docker image

```
docker build -t rewms:latest .
```

Then run with docker. When running with docker, an nginx cache reverse proxies the wms. In this scenario, the `downstream` host must be supplied to the docker image,
pointing to the ncWMS endpoint to use as the downstream wms. The example given below hits the `tds.maracoos.org` ncWMS2 instance.

```
docker run -p 80:80 --add-host=downstream:20.228.242.252 rewms:latest
```