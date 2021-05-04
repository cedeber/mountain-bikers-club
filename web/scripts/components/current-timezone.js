import { LitElement, html } from 'https://cdn.skypack.dev/lit?min'

export class CurrentTimezone extends LitElement {
  datetime

  constructor () {
    super()
    this._handleChange = this._handleChange.bind(this)
    this._toTimeZone = this._toTimeZone.bind(this)
    this.options = {
      // timeZone: 'UTC',
      timeZoneName: 'long',
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }
    this.date = new Date()
    this.diff = this.date.getTimezoneOffset()
  }

  static get properties () {
    return {
      datetime: { type: String },
      form: { type: HTMLFormElement, attribute: false },
      options: { type: Object, attribute: false },
      date: { type: Number, attribute: false },
      diff: { type: Number, attribute: false },
      day: { type: Number, attribute: false },
      month: { type: Number, attribute: false },
      year: { type: Number, attribute: false },
      hour: { type: Number, attribute: false },
      minute: { type: Number, attribute: false },
    }
  }

  createRenderRoot () {
    // will render outside of the Shadow DOM
    return this
  }

  _handleChange () {
    let data = new FormData(this.form)
    this.day = Number(data.get('day'))
    this.month = Number(data.get('month')) - 1 // Date() month starts at 0
    this.year = Number(data.get('year'))
    this.hour = Number(data.get('hour'))
    this.minute = Number(data.get('minute'))

    if (this.day > 0 && this.year > 0) {
      this.date = new Date(this.year, this.month, this.day, this.hour, this.minute)
      this.diff = this.date.getTimezoneOffset()
    }
  }

  _toTimeZone () {
    let date = new Date(Date.parse(this.datetime))

    try {
      this.form.querySelector('select[name=day]').value = date.getDate() // getDay() is monday - sunday
      this.form.querySelector('select[name=month]').value = date.getMonth() + 1 // month starts from 0
      this.form.querySelector('select[name=year]').value = date.getFullYear()
      this.form.querySelector('select[name=hour]').value = date.getHours()
      this.form.querySelector('select[name=minute]').value = date.getMinutes()
    } catch (e) { /* empty */ }

    this._handleChange()
  }

  connectedCallback () {
    super.connectedCallback()

    this.form = this.closest('form')

    if (this.form != null) {
      this.form.addEventListener('change', this._handleChange)
      // force update fields from UTC on first load
      if (this.datetime) this._toTimeZone()
    }
  }

  render () {
    return html`
      <div class="form-text">
        ${this.date.toLocaleDateString(navigator.language, this.options)}
      </div>
      <input type="hidden" name="timezone_diff" value="${this.diff}">
    `
  }
}
