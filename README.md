# **_BIG CHANGES COMING_**

- Built-in cache

# `pxcmprs-server`

## REST API documentation

### `GET /transform/:source`

`source` is the URL of the source media encoded in base64 (url-safe).

#### Formats

| Name | Extension       |
| ---- | --------------- |
| JPEG | `.jpeg`, `.jpg` |
| WebP | `.webp`         |
| PNG  | `.png`          |
| GIF  | `.gif`          |

#### Query parameters

| Parameter | Type   | Description                                                                                                |
| --------- | ------ | ---------------------------------------------------------------------------------------------------------- |
| `width`   | `?int` | Width of the new media.                                                                                    |
| `height`  | `?int` | Height of the new media.                                                                                   |
| `quality` | `?int` | Encoding quality (only used for lossy encodings like `WebP` and `JPEG`). Must be in the range of 0-100. |

#### Example

![](https://cdn.spacetelescope.org/archives/images/large/heic0206b.jpg)

This image taken by the Hubble Space Telescope is a 3857×2893 JPEG. Its size is about 2.6 MiB. To convert it to WebP, you must first encode the url ([https://cdn.spacetelescope.org/archives/images/large/heic0206b.jpg](https://cdn.spacetelescope.org/archives/images/large/heic0206b.jpg)) to base64. The URL safe variant used by pxcmprs-core results in `aHR0cHM6Ly9jZG4uc3BhY2V0ZWxlc2NvcGUub3JnL2FyY2hpdmVzL2ltYWdlcy9sYXJnZS9oZWljMDIwNmIuanBn`.

`GET /transform/aHR0cHM6Ly9jZG4uc3BhY2V0ZWxlc2NvcGUub3JnL2FyY2hpdmVzL2ltYWdlcy9sYXJnZS9oZWljMDIwNmIuanBn` returns the new image, auto-converted to either JPEG or WebP based on the client's `accept` header in order for older browsers – I'm looking at you, Internet Explorer – to be happy. If you want to force convert to `PNG`, just add a `.png` extension to the url.
