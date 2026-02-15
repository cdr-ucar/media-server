# media-server

> S3 public file server

> [!WARNING]
> This project is experimental.

## Quick start

Example configuration :

```yaml
# config.yml
buckets:
  my-media:
    endpoint_url: "https://s3.example.com"
    bucket_name: "my-bucket"
    access_key:
      plain: "<access_key>"
    secret_key:
      plain: "<secret_key>"
```

```sh
$ docker run --rm -v ./config.yml:/etc/media-server/config.yml -e RUST_LOG=info -p 8080:8080 -it ghcr.io/cdr-ucar/media-server
2026-02-14T18:34:13.588929Z  INFO media_server: listening on [::]:8080
```

```sh
$ curl -v http://localhost:8080/my-media/path/to/file.jpg
* Host localhost:8080 was resolved.
* IPv6: ::1
* IPv4: 127.0.0.1
*   Trying [::1]:8080...
* Connected to localhost (::1) port 8080
> GET /my-media/path/to/file.jpg HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/8.7.1
> Accept: */*
>
* Request completely sent off
< HTTP/1.1 302 Found
< location: https://s3.example.com/my-bucket/path/to/file.jpg?x-id=GetObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=foo&X-Amz-Date=bar&X-Amz-Expires=300&X-Amz-SignedHeaders=host&X-Amz-Signature=baz
< content-length: 0
< date: Sat, 14 Feb 2026 18:34:23 GMT
<
* Connection #0 to host localhost left intact
```
