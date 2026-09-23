import { invokeCommand } from "./transport";
import { peerSchema, type Peer } from "../contracts/peer";
import { z } from "zod";

export async function listPeers(): Promise<Peer[]> {
  const result = await invokeCommand("peers_list", undefined, z.array(peerSchema));
  return result;
}

export async function manualConnectPeer(host: string, port: number): Promise<Peer> {
  const result = await invokeCommand("peers_manual_connect", { host, port }, peerSchema);
  return result;
}

export async function setPeerTrust(
  fingerprint: string,
  isTrusted: boolean,
  autoAccept: boolean,
): Promise<void> {
  await invokeCommand(
    "peers_set_trust",
    { fingerprint, isTrusted, autoAccept },
    z.void().nullable(),
  );
}
