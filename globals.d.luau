--!strict
declare error: (message: string) -> ()
declare warn: (message: string) -> ()
declare info: (message: string) -> ()
declare debug: (message: string) -> ()

declare signal: Signal<string>
declare game: Game

export type Plugin = {
	name: string,
	description: string,
	author: string,
	version: string,
	api_version: string,
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

export type Game = {
	Store: DataStore,
}

export type DataStore = {
	SetAsync: (self: DataStore, key: any, value: any) -> (),
	GetAsync: (self: DataStore, key: any) -> any,
	UpdateAsync: (self: DataStore, key: any, update: (any) -> any) -> (),
	RemoveAsync: (self: DataStore, key: any) -> any,
}
