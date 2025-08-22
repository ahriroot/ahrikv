package v1

const (
	CmdPing         = 0x01
	CmdAuthenticate = 0x02
	CmdKeys         = 0x03
	CmdExists       = 0x04
	CmdExpire       = 0x05
	CmdSetString    = 0x11
	CmdGetString    = 0x12
	CmdDelString    = 0x13
	CmdHashSet      = 0x21
	CmdHashGet      = 0x22
	CmdHashDel      = 0x23
	CmdHashExists   = 0x24
	CmdHashLen      = 0x25
	CmdHashFields   = 0x26
	Error           = 0xff
)
