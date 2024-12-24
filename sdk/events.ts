import { BorshCoder, EventParser, Idl } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";

import idl from "../target/idl/vaults.json";
import {
  DepositMadeEvent,
  TokenWhitelistedEvent,
  VaultCreatedEvent,
  VaultFeesUpdateRequestedEvent,
  VaultFeesUpdatedEvent,
  WithdrawMadeEvent,
} from "./types";

export class EventHandler {
  private connection: Connection;
  private programId: PublicKey;
  private parser: EventParser;

  constructor(connection: Connection, programId: PublicKey) {
    this.connection = connection;
    this.programId = programId;
    this.parser = new EventParser(programId, new BorshCoder(idl as Idl));
  }

  async subscribeVaultCreated(callback: (event: VaultCreatedEvent) => void) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "VaultCreated")
          .forEach((event) => callback(event.data as VaultCreatedEvent));
      }
    });
  }

  async subscribeVaultFeesUpdateRequested(
    callback: (event: VaultFeesUpdateRequestedEvent) => void
  ) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "VaultFeesUpdateRequested")
          .forEach((event) =>
            callback(event.data as VaultFeesUpdateRequestedEvent)
          );
      }
    });
  }

  async subscribeVaultFeesUpdated(
    callback: (event: VaultFeesUpdatedEvent) => void
  ) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "VaultFeesUpdated")
          .forEach((event) => callback(event.data as VaultFeesUpdatedEvent));
      }
    });
  }

  async subscribeDepositMade(callback: (event: DepositMadeEvent) => void) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "DepositMade")
          .forEach((event) => callback(event.data as DepositMadeEvent));
      }
    });
  }

  async subscribeWithdrawMade(callback: (event: WithdrawMadeEvent) => void) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "WithdrawMade")
          .forEach((event) => callback(event.data as WithdrawMadeEvent));
      }
    });
  }

  async subscribeTokenWhitelisted(
    callback: (event: TokenWhitelistedEvent) => void
  ) {
    this.connection.onLogs(this.programId, (logs) => {
      if (!logs.err) {
        const events = Array.from(this.parser.parseLogs(logs.logs));
        events
          .filter((event) => event.name === "TokenWhitelisted")
          .forEach((event) => callback(event.data as TokenWhitelistedEvent));
      }
    });
  }
}
