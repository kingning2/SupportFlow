import { createSlice } from "@reduxjs/toolkit";

import { appConfig } from "../../../config/app-config";
import { ReduxSlice } from "@supportflow/shared/tauri-bridge/enums";

import type { AppInitialState } from "./types";
import type { IAction } from "../../types";

const initialState: AppInitialState = {
  initialized: false,
  titleBarHeight: appConfig.titleBarHeight,
  mainWindowGlobalGg: appConfig.mainWindowGlobalGg
};

const appSlice = createSlice({
  name: ReduxSlice.App,
  initialState,
  reducers: {
    changeInitializedAction(state, { payload }: IAction<AppInitialState["initialized"]>) {
      state.initialized = payload;
    },
    changeMainWindowGlobalGgAction(
      state,
      { payload }: IAction<AppInitialState["mainWindowGlobalGg"]>
    ) {
      state.mainWindowGlobalGg = payload;
    }
  }
});

export const { changeInitializedAction, changeMainWindowGlobalGgAction } = appSlice.actions;

export default appSlice.reducer;
