import 'package:veilid/veilid.dart';

class ProcessorConnectionState {
  var attachment = VeilidStateAttachment(
      localNetworkReady: false,
      publicInternetReady: false,
      state: AttachmentState.detached,
      uptime: TimestampDuration(
        value: BigInt.zero,
      ),
      attachedUptime: null);

  var network = VeilidStateNetwork(
      bpsDown: BigInt.zero, bpsUp: BigInt.zero, started: false, peers: []);

  ProcessorConnectionState();

  bool get isAttached => !(attachment.state == AttachmentState.detached ||
      attachment.state == AttachmentState.detaching ||
      attachment.state == AttachmentState.attaching);

  bool get isPublicInternetReady => attachment.publicInternetReady;
}
