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

Then run with docker 

```
docker run -p 8080:8080 --env PORT=8080 --env DOWNSTREAM="eds.ioos.us" rewms:latest
```