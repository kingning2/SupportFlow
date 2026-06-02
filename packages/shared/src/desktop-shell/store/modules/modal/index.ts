import { createSlice } from "@reduxjs/toolkit";

import { ReduxSlice } from "@supportflow/shared/tauri-bridge/enums";

import type { ModalInitialState } from "./types";
import type { IAction } from "../../types";

const initialState: ModalInitialState = {
  openLabels: []
};

const modalSlice = createSlice({
  name: ReduxSlice.Modal,
  initialState,
  reducers: {
    modalOpenedAction(state, { payload }: IAction<string>) {
      if (!state.openLabels.includes(payload)) {
        state.openLabels.push(payload);
      }
    },
    modalClosedAction(state, { payload }: IAction<string>) {
      state.openLabels = state.openLabels.filter((label) => label !== payload);
    }
  }
});

export const { modalOpenedAction, modalClosedAction } = modalSlice.actions;

export default modalSlice.reducer;
