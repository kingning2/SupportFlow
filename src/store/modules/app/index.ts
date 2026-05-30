import { createSlice } from "@reduxjs/toolkit";

import { appConfig } from "@/config/app-config";
import { LocalCacheKey, ReduxSlice } from "@/enums";
import { localCache } from "@/utils/cache";

import type { AppInitialState } from "./types";
import type { IAction } from "@/store/types";

const initialState: AppInitialState = {
  initialized: false,
  titleBarHeight: appConfig.titleBarHeight,
  mainWindowGlobalGg: appConfig.mainWindowGlobalGg,
  supportLanguages: appConfig.supportLanguages,
  currentLanguage:
    (localCache.getCache(LocalCacheKey.Language) as AppInitialState["currentLanguage"]) ??
    appConfig.defaultLanguage
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
    },
    changeCurrentLanguageAction(state, { payload }: IAction<AppInitialState["currentLanguage"]>) {
      state.currentLanguage = payload;
      localCache.setCache(LocalCacheKey.Language, payload);
    }
  }
});

export const {
  changeInitializedAction,
  changeMainWindowGlobalGgAction,
  changeCurrentLanguageAction
} = appSlice.actions;

export default appSlice.reducer;
