import { configureStore } from "@reduxjs/toolkit";

import appReducer from "./modules/app";
import modalReducer from "./modules/modal";

const makeStore = () => {
  return configureStore({
    reducer: {
      app: appReducer,
      modal: modalReducer
    }
  });
};

export default makeStore;

export type AppStore = ReturnType<typeof makeStore>;
export type RootState = ReturnType<AppStore["getState"]>;
export type AppDispatch = AppStore["dispatch"];
