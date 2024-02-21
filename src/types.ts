import Afip from '@afipsdk/afip.js'

export interface Cert {
  cert: string
  key: string
}

export interface AfipExtended extends Afip {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  CreateWSAuth(username: string, password: string, alias: string, wsid: string): Promise<any>

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  CreateCert(username: string, password: string, alias: string): Promise<Cert>
}
