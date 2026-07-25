--!strict
export type Plugin = {
	on_enable: () -> (),
	on_disable: () -> (),
}
