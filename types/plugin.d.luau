--!strict
export type Plugin = {
	name: string,
	description: string,
	author: string,
	version: string,
	api_version: string,
	on_enable: () -> (),
	on_disable: () -> (),
}
