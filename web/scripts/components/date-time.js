import { LitElement, html } from 'https://cdn.skypack.dev/lit-element?min'

export class DateTime extends LitElement {
  short

  static get properties () {
    return {
      date: { type: String },
      short: { type: Boolean },
    }
  }

  constructor () {
    super()
  }

  connectedCallback () {
    super.connectedCallback()

    this.options = this.short === true ? {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    } : {
      // timeZone: 'UTC',
      timeZoneName: 'long',
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }
  }

  render () {
    let date = new Date(Date.parse(this.date))
    return html`
      <span lang="${navigator.language}">
        ${date.toLocaleDateString(navigator.language, this.options)}
      </span>
    `
  }
}
