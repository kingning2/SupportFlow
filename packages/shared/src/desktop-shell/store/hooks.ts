import { useDispatch, useSelector, useStore } from "react-redux";
import { createSelector } from "@reduxjs/toolkit";

import type { RootState, AppDispatch, AppStore } from "./index";

export const useAppDispatch = useDispatch.withTypes<AppDispatch>();
export const useAppSelector = useSelector.withTypes<RootState>();
export const useAppStore = useStore.withTypes<AppStore>();
export const useAppCreateSelector = createSelector.withTypes<RootState>();
