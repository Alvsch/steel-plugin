--!strict
declare log: Log
declare signal: Signal<string>

export type Plugin = {
	name: string,
	description: string,
	version: string,
	author: string,
	on_enable: () -> (),
	on_disable: () -> (),
}

export type Log = {
	Info: (message: string) -> (),
	Warn: (message: string) -> (),
	Error: (message: string) -> (),
	Debug: (message: string) -> (),
}

export type Signal<T> = {
	Connect: (self: Signal<T>, (T) -> ()) -> Connection,
	Once: (self: Signal<T>, (T) -> ()) -> Connection,
	Wait: (self: Signal<T>) -> (),
}

export type Connection = {
	Disconnect: () -> (),
	Connected: boolean,
}
