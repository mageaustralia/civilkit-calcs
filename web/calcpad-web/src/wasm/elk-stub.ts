// calcpad-rs: stands in for elkjs, the optional "elk" layout engine of Mermaid (EPL-2.0), which is
// not shipped (vite.config.ts). A diagram that asks for that layout shows its source and this reason.
export default class ElkStub {
    layout(): Promise<never> {
        return Promise.reject(new Error('the elk layout is not available here; leave out "layout: elk" to use the default layout'));
    }
}
