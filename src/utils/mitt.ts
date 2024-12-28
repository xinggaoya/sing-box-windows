import mitt from 'mitt'
import type { Emitter } from 'mitt'

type Events = {
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
}

const emitter: Emitter<Events> = mitt<Events>()
export default emitter 