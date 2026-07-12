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

export type Game = {
	Store: DataStore,
}
