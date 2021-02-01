import L from 'https://cdn.skypack.dev/leaflet'

const Fullscreen = L.Control.extend({
  options: {
    position: 'topleft',
    title: {
      'false': 'View Fullscreen',
      'true': 'Exit Fullscreen',
    },
  },

  onAdd: function (map) {
    const container = L.DomUtil.create('div', 'leaflet-control-fullscreen leaflet-bar leaflet-control')

    this.link = L.DomUtil.create('a', 'leaflet-control-fullscreen-button leaflet-bar-part', container)
    this.link.href = '#'

    this._map = map
    this._map.on('fullscreenchange', this._toggleTitle, this)
    this._toggleTitle()

    L.DomEvent.on(this.link, 'click', this._click, this)

    return container
  },

  _click: function (e) {
    L.DomEvent.stopPropagation(e)
    L.DomEvent.preventDefault(e)
    this._map.toggleFullscreen(this.options)
  },

  _toggleTitle: function () {
    this.link.title = this.options.title[this._map.isFullscreen()]
  },
})

L.Map.include({
  isFullscreen: function () {
    return this._isFullscreen || false
  },

  toggleFullscreen: function () {
    const container = this.getContainer()

    if (this.isFullscreen()) {
      this._disablePseudoFullscreen(container)
    } else {
      this._enablePseudoFullscreen(container)
    }

  },

  _enablePseudoFullscreen: function (container) {
    L.DomUtil.addClass(container, 'leaflet-pseudo-fullscreen')
    this._setFullscreen(true)
    this.fire('fullscreenchange')
  },

  _disablePseudoFullscreen: function (container) {
    L.DomUtil.removeClass(container, 'leaflet-pseudo-fullscreen')
    this._setFullscreen(false)
    this.fire('fullscreenchange')
  },

  _setFullscreen: function (fullscreen) {
    const container = this.getContainer()

    this._isFullscreen = fullscreen

    if (fullscreen) {
      L.DomUtil.addClass(container, 'leaflet-fullscreen-on')
    } else {
      L.DomUtil.removeClass(container, 'leaflet-fullscreen-on')
    }

    this.invalidateSize()
  },
})

L.Map.mergeOptions({
  fullscreenControl: false,
})

L.Map.addInitHook(function () {
  if (this.options.fullscreenControl) {
    this.fullscreenControl = new Fullscreen(this.options.fullscreenControl)
    this.addControl(this.fullscreenControl)
  }
})