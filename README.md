# rewms

Restyle WMS raster images to optimize them for webgl rendering

## Building and running

### With Cargo

To run against the IOOS EDS WMS:

```
cargo run -- --wms-root="https://eds.ioos.us/ncWMS2" --port=9080
```

You can install the app as a binary with 

```
cargo install --path .
```

This will install the statically linked binary to your cargo bin directory, where it can be run or copied elsewhere on your system for use.

### With Docker

First build the docker image

```
docker build -t rewms:latest .
```

Then run with docker. 

```
docker run -p 80:9080 rewms:latest --port=9080 --wms-root="https://eds.ioos.us/ncWMS2"
```

### With Docker and NGINX

First build the docker image

```
docker build -f Dockerfile.nginx -t rewms:latest .
```

Then run with docker. With this image, a nginx cache reverse proxies the wms. In this scenario, the `downstream_wms` host must be supplied to the docker image, pointing to the ncWMS endpoint to use as the downstream wms. The example given below hits the `tds.maracoos.org` ncWMS2 instance.

```
docker run -p 80:80 --add-host=downstream_wms:20.228.242.252 rewms:latest
```

NOTE: For now, the downstream_wms host is expected to be https. If its, not, edit the nginx default.conf to reflect http instead

## CLI options

**wms-root *(REQUIRED)*:** The path the the `ncWMS` root url to use as the downstream server. *EX:* `--wms-root=https://eds.ioos.us/ncWMS2`

**port**: The port to bind the server to. *EX:* `--port=9080`. *DEFAULT:* `9080`

**workers**: The number of worker threads to run. This is limited byt he number of available cores on the target machine. *EX:* `--workers=6`. *DEFAULT:* `1`