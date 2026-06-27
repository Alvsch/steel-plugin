--!strict
declare error: (message: string) -> ()
declare warn: (message: string) -> ()
declare info: (message: string) -> ()
declare debug: (message: string) -> ()

declare signal: Signal<string>

export type Plugin = {
	name: string,
	description: string,
	version: string,
	author: string,
	on_enable: () -> (),
	on_disable: () -> (),
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
