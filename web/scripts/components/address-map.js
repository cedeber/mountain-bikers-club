import { LitElement, html } from 'https://cdn.skypack.dev/lit-element?min'
import L from 'https://cdn.skypack.dev/leaflet?min'

export class AddressMap extends LitElement {
  static get properties () {
    return {
      address: { type: String },
      zoom: { type: Number },
    }
  }

  constructor () {
    super()
    this.position = undefined
  }

  render () {
    return html`
      <link rel="stylesheet" href="https://cdn.skypack.dev/leaflet/dist/leaflet.css">
      <link rel="stylesheet" href="/web/assets/fonts/fa.css">
      <div id="map" style="width: 100%; height: 200px;"></div>
    `
  }

  firstUpdated (changedProperties) {
    let map = this.shadowRoot.querySelector('#map')

    fetch(`https://photon.komoot.io/api/?q=${this.address}&limit=1`).then(response => response.json()).then(result => {
      if (Array.isArray(result.features) && result.features.length > 0 && result.features[0].geometry && result.features[0].geometry.coordinates) {
        this.position = result.features[0].geometry.coordinates
        map = L.map(map, {
          scrollWheelZoom: false,
          zoomControl: false, // self designed
          attributionControl: false,
          doubleClickZoom: false,
          dragging: false,
        }).setView([this.position[1], this.position[0]], this.zoom)

        L.control.zoom({
          position: 'bottomright',
          zoomInText: '<i class="fad fa-search-plus" style="text-indent: 0; font-size: 20px"></i>',
          zoomOutText: '<i class="fad fa-search-minus" style="text-indent: 0; font-size: 20px"></i>',
        }).addTo(map)

        // https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png
        // https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png
        // https://tile.thunderforest.com/cycle/{z}/{x}/{y}.png?apikey={}
        // http://{s}.tile.komoot.de/komoot-2/{z}/{x}/{y}.png
        // 'https://{s}.tiles-api.maps.komoot.net/v1/tiles/osm-optimized/{z}/{x}/{y}.vector.pbf
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
          attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors,' +
            '<a href="http://viewfinderpanoramas.org">SRTM</a>, <a href="https://opentopomap.org">OpenTopoMap</a>',
          // maxZoom: 17,
        }).addTo(map)

        L.circle([this.position[1], this.position[0]], {
          color: 'red',
          fillColor: 'crimson',
          fillOpacity: 0.5,
          radius: 60
        }).addTo(map)
      }
    })
  }
}
