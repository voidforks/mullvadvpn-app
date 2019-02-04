import * as React from 'react';
import { Animated, Button, Component, Text, Types, View, Styles, UserInterface } from 'reactxp';
import { ImageView } from '@mullvad/components';
import CustomScrollbars, { ScrollEvent } from './CustomScrollbars';
import { colors } from '../../config.json';

const styles = {
  navigationBar: {
    default: Styles.createViewStyle({
      flex: 0,
      flexDirection: 'row',
      paddingHorizontal: 12,
      paddingBottom: 12,
    }),
    separator: Styles.createViewStyle({
      borderStyle: 'solid',
      borderBottomWidth: 1,
      borderColor: 'rgba(0, 0, 0, 0.2)',
    }),
    darwin: Styles.createViewStyle({
      paddingTop: 24,
    }),
    win32: Styles.createViewStyle({
      paddingTop: 12,
    }),
    linux: Styles.createViewStyle({
      paddingTop: 12,
      appRegion: 'drag',
    }),
  },
  navigationBarTitle: {
    container: Styles.createViewStyle({
      flex: 1,
      flexDirection: 'column',
      justifyContent: 'center',
    }),
    label: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 16,
      fontWeight: '600',
      lineHeight: 22,
      color: colors.white,
      alignSelf: 'center',
    }),
  },
  closeBarItem: {
    default: Styles.createViewStyle({
      cursor: 'default',
      appRegion: 'no-drag',
    }),
    icon: Styles.createViewStyle({
      flex: 0,
      opacity: 0.6,
    }),
  },
  backBarButton: {
    default: Styles.createViewStyle({
      borderWidth: 0,
      padding: 0,
      margin: 0,
      cursor: 'default',
      appRegion: 'no-drag',
    }),
    content: Styles.createViewStyle({
      flexDirection: 'row',
      alignItems: 'center',
    }),
    label: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: colors.white60,
    }),
    icon: Styles.createViewStyle({
      opacity: 0.6,
      marginRight: 8,
    }),
  },
};

type NavigationScrollContextValue = {
  scrollTop: number;
  onScroll: (event: ScrollEvent) => void;
};

const NavigationScrollContext = React.createContext<NavigationScrollContextValue>({
  scrollTop: 0,
  onScroll: (_event: ScrollEvent) => {},
});

export class NavigationContainer extends Component {
  state = {
    scrollTop: 0,
  };

  _onScroll = (event: ScrollEvent) => {
    this.setState({
      scrollTop: event.scrollTop,
    });
  };

  render() {
    return (
      <NavigationScrollContext.Provider
        value={{ scrollTop: this.state.scrollTop, onScroll: this._onScroll }}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }
}

type NavigationScrollbarsProps = {
  onScroll?: (value: ScrollEvent) => void;
  style?: React.CSSProperties;
  children?: React.ReactNode;
};
export const NavigationScrollbars = React.forwardRef(function NavigationScrollbarsT(
  props: NavigationScrollbarsProps,
  ref?: React.Ref<CustomScrollbars>,
) {
  return (
    <NavigationScrollContext.Consumer>
      {(context) => {
        const { style, children, ...otherProps } = props;
        const wrappedOnScroll = (scroll: ScrollEvent) => {
          context.onScroll(scroll);

          if (otherProps.onScroll) {
            otherProps.onScroll(scroll);
          }
        };

        return (
          <CustomScrollbars ref={ref} style={style} onScroll={wrappedOnScroll}>
            {children}
          </CustomScrollbars>
        );
      }}
    </NavigationScrollContext.Consumer>
  );
});

type PrivateTitleBarItemProps = {
  visible: boolean;
  titleAdjustment: number;
  children?: React.ReactText;
};

class PrivateTitleBarItem extends Component<PrivateTitleBarItemProps> {
  shouldComponentUpdate(nextProps: PrivateTitleBarItemProps) {
    return (
      this.props.visible !== nextProps.visible ||
      this.props.titleAdjustment !== nextProps.titleAdjustment ||
      this.props.children !== nextProps.children
    );
  }

  render() {
    const titleAdjustment = this.props.titleAdjustment;
    const titleAdjustmentStyle = Styles.createViewStyle(
      {
        paddingRight: titleAdjustment > 0 ? titleAdjustment : 0,
        paddingLeft: titleAdjustment < 0 ? Math.abs(titleAdjustment) : 0,
      },
      false,
    );

    return (
      <View style={[styles.navigationBarTitle.container, titleAdjustmentStyle]}>
        <PrivateBarItemAnimationContainer visible={this.props.visible}>
          <Text style={styles.navigationBarTitle.label}>{this.props.children}</Text>
        </PrivateBarItemAnimationContainer>
      </View>
    );
  }
}

type PrivateBarItemAnimationContainerProps = {
  visible: boolean;
  children?: React.ReactNode;
};

class PrivateBarItemAnimationContainer extends Component<PrivateBarItemAnimationContainerProps> {
  _opacityValue: Animated.Value;
  _opacityStyle: Types.AnimatedViewStyleRuleSet;
  _animation?: Types.Animated.CompositeAnimation;

  constructor(props: PrivateBarItemAnimationContainerProps) {
    super(props);

    this._opacityValue = Animated.createValue(props.visible ? 1 : 0);
    this._opacityStyle = Styles.createAnimatedViewStyle({
      opacity: this._opacityValue,
    });
  }

  shouldComponentUpdate(nextProps: PrivateBarItemAnimationContainerProps) {
    return this.props.visible !== nextProps.visible || this.props.children !== nextProps.children;
  }

  componentDidUpdate() {
    this._animateOpacity(this.props.visible);
  }

  componentWillUnmount() {
    if (this._animation) {
      this._animation.stop();
    }
  }

  render() {
    return <Animated.View style={this._opacityStyle}>{this.props.children}</Animated.View>;
  }

  _animateOpacity(visible: boolean) {
    const oldAnimation = this._animation;
    if (oldAnimation) {
      oldAnimation.stop();
    }

    const animation = Animated.timing(this._opacityValue, {
      toValue: visible ? 1 : 0,
      easing: Animated.Easing.InOut(),
      duration: 250,
    });

    animation.start();

    this._animation = animation;
  }
}

type NavigationBarProps = {
  children?: React.ReactNode;
};

export const NavigationBar = React.forwardRef(function NavigationBarT(
  props: NavigationBarProps,
  ref?: React.Ref<PrivateNavigationBar>,
) {
  return (
    <NavigationScrollContext.Consumer>
      {(context) => (
        <PrivateNavigationBar ref={ref} scrollTop={context.scrollTop}>
          {props.children}
        </PrivateNavigationBar>
      )}
    </NavigationScrollContext.Consumer>
  );
});

type PrivateNavigationBarProps = {
  scrollTop: number;
  children?: React.ReactNode;
};

type PrivateNavigationBarState = {
  titleAdjustment: number;
  showsBarSeparator: boolean;
  showsBarTitle: boolean;
};

const PrivateTitleBarItemContext = React.createContext({
  titleAdjustment: 0,
  visible: false,
  titleRef: React.createRef<PrivateTitleBarItem>(),
});

class PrivateNavigationBar extends Component<PrivateNavigationBarProps, PrivateNavigationBarState> {
  static defaultProps: Partial<PrivateNavigationBarProps> = {
    scrollTop: 0,
  };

  state: PrivateNavigationBarState = {
    titleAdjustment: 0,
    showsBarSeparator: false,
    showsBarTitle: false,
  };

  _titleViewRef = React.createRef<PrivateTitleBarItem>();

  static getDerivedStateFromProps(
    props: PrivateNavigationBarProps,
    state: PrivateNavigationBarState,
  ) {
    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    const showsBarSeparator = props.scrollTop > 11;

    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    const showsBarTitle = props.scrollTop > 30;

    return {
      ...state,
      showsBarSeparator,
      showsBarTitle,
    };
  }

  shouldComponentUpdate(
    nextProps: PrivateNavigationBarProps,
    nextState: PrivateNavigationBarState,
  ) {
    return (
      this.props.children !== nextProps.children ||
      this.state.titleAdjustment !== nextState.titleAdjustment ||
      this.state.showsBarSeparator !== nextState.showsBarSeparator ||
      this.state.showsBarTitle !== nextState.showsBarTitle
    );
  }

  render() {
    return (
      <View
        style={[
          styles.navigationBar.default,
          this.state.showsBarSeparator ? styles.navigationBar.separator : undefined,
          this._getPlatformStyle(),
        ]}
        onLayout={this._onLayout}>
        <PrivateTitleBarItemContext.Provider
          value={{
            titleAdjustment: this.state.titleAdjustment,
            visible: this.state.showsBarTitle,
            titleRef: this._titleViewRef,
          }}>
          {this.props.children}
        </PrivateTitleBarItemContext.Provider>
      </View>
    );
  }

  _getPlatformStyle(): Types.ViewStyleRuleSet | undefined {
    switch (process.platform) {
      case 'darwin':
        return styles.navigationBar.darwin;
      case 'win32':
        return styles.navigationBar.win32;
      case 'linux':
        return styles.navigationBar.linux;
      default:
        return undefined;
    }
  }

  _onLayout = async (containerLayout: Types.ViewOnLayoutEvent) => {
    const titleView = this._titleViewRef.current;
    if (titleView) {
      // calculate the title layout frame
      const titleLayout = await UserInterface.measureLayoutRelativeToAncestor(titleView, this);

      // calculate the remaining space at the right hand side
      const trailingSpace = containerLayout.width - (titleLayout.x + titleLayout.width);

      this.setState({
        titleAdjustment: titleLayout.x - trailingSpace,
      });
    }
  };
}

type TitleBarItemProps = {
  children?: React.ReactText;
};
export function TitleBarItem(props: TitleBarItemProps) {
  return (
    <PrivateTitleBarItemContext.Consumer>
      {(context) => (
        <PrivateTitleBarItem
          titleAdjustment={context.titleAdjustment}
          visible={context.visible}
          ref={context.titleRef}>
          {props.children}
        </PrivateTitleBarItem>
      )}
    </PrivateTitleBarItemContext.Consumer>
  );
}

export class CloseBarItem extends Component<{
  action: () => void;
}> {
  render() {
    return (
      <Button style={[styles.closeBarItem.default]} onPress={this.props.action}>
        <ImageView height={24} width={24} style={[styles.closeBarItem.icon]} source="icon-close" />
      </Button>
    );
  }
}

export class BackBarItem extends Component<{
  children?: React.ReactText;
  action: () => void;
}> {
  render() {
    return (
      <Button style={styles.backBarButton.default} onPress={this.props.action}>
        <View style={styles.backBarButton.content}>
          <ImageView style={styles.backBarButton.icon} source="icon-back" />
          <Text style={styles.backBarButton.label}>{this.props.children}</Text>
        </View>
      </Button>
    );
  }
}