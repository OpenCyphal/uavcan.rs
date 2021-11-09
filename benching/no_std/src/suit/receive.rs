use embedded_time::Clock;
use uavcan::{
    session::SessionManager,
    transport::can::{Can, CanFrame, CanMessageId, FakePayloadIter},
    types::PortId,
    Node,
};

use super::{
    context::{NeedsClock, NeedsNode},
    Bencher,
};

#[optimize(speed)]
fn receive<S: SessionManager<C>, C: embedded_time::Clock + 'static + Clone>(
    node: &mut Node<S, Can, C>,
    frame: CanFrame<C>,
) -> bool {
    if let Some(frame) = node.try_receive_frame(frame).unwrap() {
        core::hint::black_box(frame);
        return true;
    }
    false
}

pub fn bench_receive<Context, CM: embedded_time::Clock, const N: usize>(
    bencher: &mut Bencher<CM>,
    context: &mut Context,
) where
    Context: NeedsNode<TransportType = Can> + NeedsClock,
{
    let port_id: PortId = 7168;
    let message_id = CanMessageId::new(uavcan::Priority::Immediate, port_id, Some(1));
    let mut transfer_id = 0u8;

    bencher.run_with_watch(|watch| {
        let payload_iter = FakePayloadIter::<8>::multi_frame(N, transfer_id);
        for payload in payload_iter {
            let payload = arrayvec::ArrayVec::from_iter(payload);
            let frame = core::hint::black_box(CanFrame {
                id: message_id,
                payload,
                timestamp: context.clock_as_mut().try_now().unwrap(),
            });
            watch.start();
            if receive(context.node_as_mut(), core::hint::black_box(frame)) {
                watch.stop();
                break;
            }
            watch.stop();
            transfer_id = transfer_id.wrapping_add(1);
        }
    })
}
