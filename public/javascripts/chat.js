$(document).ready(function (e) {
  const $chatBox = $("#input_content");
  const $userList = $("#userList");

  $chatBox.on("input", function () {
    const inputVal = $chatBox.text();
    var users = window.users;

    for (var i = 0; i < users.length; i++) {
      if (users[i] == undefined) {
        users[i] = "null"
      }
    }
    //console.log(users);

    const atIndex = inputVal.lastIndexOf("@");
    if (atIndex !== -1) {
      const query = inputVal.substring(atIndex + 1);
      const filteredUsers = users.filter((user) =>
        user.toLowerCase().includes(query.toLowerCase())
      );
      $userList.empty();
      filteredUsers.forEach((user) => {
        $userList.append(`<li>${user}</li>`);
      });
      const chatBoxOffset = $chatBox.offset();
      $userList
        .css({
          top: chatBoxOffset.top - 300 + $chatBox.outerHeight(),
          left: chatBoxOffset.left + 30,
        })
        .show();
    } else {
      $userList.hide();
    }
  });

  $userList.on("click", "li", function () {
    const selectedUser = $(this).text();
    const inputVal = $chatBox.text();
    const atIndex = inputVal.lastIndexOf("@");
    const newVal = inputVal.substring(0, atIndex + 1) + selectedUser + "　";
    console.log(newVal);
    $chatBox.text(newVal);
    $userList.hide();
    placeCaretAtEnd($chatBox[0]);
  });

  function placeCaretAtEnd(el) {
    el.focus();
    if (
      typeof window.getSelection != "undefined" &&
      typeof document.createRange != "undefined"
    ) {
      const range = document.createRange();
      range.selectNodeContents(el);
      range.collapse(false);
      const sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    }
  }

  $(document).on("click", function (e) {
    if (!$(e.target).closest("#chatBox, #userList").length) {
      $userList.hide();
    }
  });

  $(window).keydown(function (e) {
    if (e.keyCode == 116) {
      if (!confirm("刷新会将所有数据情况，确定要刷新么？")) {
        e.preventDefault();
      }
    }
  });
  var from = $.cookie("user");
  var to = "all";
  window.reconnectd = false;
  $("#input_content").html("");
  if (/Firefox\/\s/.test(navigator.userAgent)) {
    var socket = io.connect({ transports: ["xhr-polling"] });
  } else if (/MSIE (\d+.\d+);/.test(navigator.userAgent)) {
    var socket = io.connect({ transports: ["jsonp-polling"] });
  } else {
    var socket = io.connect();
  }
  socket.emit("online", JSON.stringify({ user: from }));
  socket.on("disconnect", function () {
    var msg = '<div style="color:#f00">SYSTEM:连接服务器失败</div>';
    addMsg(msg);
    $("#list").empty();
  });
  socket.on("reconnect", function () {
    socket.emit("online", JSON.stringify({ user: from }));
    var msg = '<div style="color:#f00">SYSTEM:重新连接服务器</div>';
    window.reconnectd = true;
    addMsg(msg);
    //经过5秒后再将reconnectd设置成false
    setTimeout(function () {
      window.reconnectd = false;
    }, 8000);
  });
  socket.on("notice", function (data) {
    var data = JSON.parse(data);
    var time = getTimeShow(data.time);
    var msg = data.msg;
    // 添加 white-space: pre-wrap; 样式
    var msg = '<div style="color:#f00; white-space: pre-wrap;">SYSTEM(' + time + "):<br/>" + msg + "</div>";
    addMsg(msg);
    play_ring("/ring/online.wav");
  });
  socket.on("system", function (data) {
    var data = JSON.parse(data);
    var time = getTimeShow(data.time);
    var msg = "";
    if (!window.reconnectd) {
      //重连成功后的通知不显示
      if (data.type == "online") {
        msg += "用户 " + data.msg + " 上线了！";
      } else if (data.type == "offline") {
        msg += "用户 " + data.msg + " 下线了！";
      } else if (data.type == "in") {
        msg += "你进入了聊天室！";
      } else {
        msg += "未知系统消息！";
      }
      var msg =
        '<div style="color:#f00">SYSTEM(' + time + "):" + msg + "</div>";
      addMsg(msg);
      play_ring("/ring/online.wav");
    }
  });
  socket.on("userflush", function (data) {
    var data = JSON.parse(data);
    var users = data.users;
    window.users = users;
    flushUsers(users);
  });

  socket.on("clear_history_client", function () {
    $("#contents").empty();
    console.log("服务器下发了清空指令，已清空本地面板");
  });

  socket.on("image", (data) => {
    console.log(data);
    var data = JSON.parse(data);
    var time = getTimeShow(new Date());
    var imageData = data.msg;
    var src = `data:image/png;base64,${imageData}`;
    var msg =
      "<div>" +
      data.from +
      "(" +
      time +
      ")上图：<br/><img onclick='popupImg(this.src);return false;' src='" +
      src +
      "' style='max-width: 300px; max-height: 300px;'></img></div>";
    addMsg(msg);
  });

  socket.on("say", function (msgData) {
    var time = msgData.time;
    time = getTimeShow(time);
    var data = msgData.data;
    console.log(msgData);
    if (data.to == "all") {
      addMsg(
        "<div>" + data.from + "(" + time + ")说：<br/>" + data.msg + "</div>"
      );
      //防止重连后疯狂通知
      if (data.from != from && !window.reconnectd && msgData.notice) {
        showNotice(data.from + "：" + data.msg);
        play_ring("/ring/msg.wav");
      }
    } else if (data.from == from) {
      addMsg(
        "<div>我(" + time + ")对" + data.to + "说：<br/>" + data.msg + "</div>"
      );
    } else if (data.to == from) {
      addMsg(
        "<div>" +
        data.from +
        "(" +
        time +
        ")对我说：<br/>" +
        data.msg +
        "</div>"
      );
      if (!window.reconnectd && msgData.notice) {
        showNotice(data.from + "说：" + data.msg);
        play_ring("/ring/msg.wav");
      }
    }
    $("img").click(function () {
      popupImg(this.src);
      return false;
    });
  });

  function showNotice(msg) {
    //发送通知
    newNotify = function () {
      var notification = new Notification("系统通知:", {
        dir: "auto",
        lang: "hi",
        requireInteraction: false,
        //tag: "testTag",
        icon: "",
        body: msg,
      });
      notification.onclick = function (event) {
        //回到发送此通知的页面
        window.focus();
        //回来后要做什么
        console.log("I'm back");
      };
    };
    //权限判断
    if (Notification.permission == "granted") {
      newNotify();
    } else {
      //请求权限
      Notification.requestPermission(function (perm) {
        if (perm == "granted") {
          newNotify();
        }
      });
    }
  }

  function addMsg(msg) {
    $("#contents").append(msg);
    $("#contents").append("<br/>");
    $("#contents").scrollTop($("#contents")[0].scrollHeight);
  }
  function flushUsers(users) {
    var ulEle = $("#list");
    ulEle.empty();
    ulEle.append(
      '<li title="双击聊天" alt="all" onselectstart="return false">所有人</li>'
    );
    for (var i = 0; i < users.length; i++) {
      ulEle.append(
        '<li alt="' +
        users[i] +
        '" title="双击聊天" onselectstart="return false">' +
        users[i] +
        "</li>"
      );
    }
    //双击对某人聊天
    $("#list > li").dblclick(function (e) {
      if ($(this).attr("alt") != from) {
        to = $(this).attr("alt");
        show_say_to();
      }
    });
    show_say_to();
  }
  $("#input_content").keydown(function (e) {
    if (e.shiftKey && e.which == 13) {
      $("#input_content").append("<br/>");
    } else if (e.which == 13) {
      e.preventDefault();
      say();
    }
  });
  $("#say").click(function (e) {
    say();
  });
  $("#clear_btn").click(function (e) {
    if (confirm("确定要清空所有聊天和播报记录吗？")) {
      socket.emit("clear_history");
    }
    $(this).blur(); // 强制失去焦点，恢复样式
  });
  $("#image-input").change(function (e) {
    const file = e.target.files[0];

    if (file) {
      const reader = new FileReader();
      if (file.type == "image/png") {
        console.log("发送图片", file);
        reader.onload = (event) => {
          socket.emit(
            "image",
            JSON.stringify({
              to: to,
              from: from,
              msg: event.target.result.split(",")[1],
            })
          );
        };
        reader.readAsDataURL(file);
      } else {
        reader.onload = (event) => {
          console.log("发送文件", file);
          const arrayBuffer = event.target.result;
          socket.emit('fileUpload', { fileName: file.name, fileBuffer: arrayBuffer });
          socket.emit(
            "say",
            JSON.stringify({
              to: to,
              from: from,
              msg: "发送文件:<a target='_blank' href='/doc/" + file.name + "'>" + file.name + "</a>",
            })
          );
        };
        reader.readAsArrayBuffer(file);
      }
    }
  });
  function say() {
    if ($("#input_content").html() == "") {
      return;
    }
    socket.emit(
      "say",
      JSON.stringify({ to: to, from: from, msg: $("#input_content").html() })
    );
    $("#input_content").html("");
    $("#input_content").focus();
  }
  function sendimg() {
    socket.emit(
      "image",
      JSON.stringify({ to: to, from: from, msg: $("#input_content").html() })
    );
    $("#input_content").html("");
    $("#input_content").focus();
  }
  //显示正在对谁说话
  function show_say_to() {
    $("#from").html(from);
    $("#to").html(to == "all" ? "所有人" : to);
    var users = $("#list > li");
    for (var i = 0; i < users.length; i++) {
      if ($(users[i]).attr("alt") == to) {
        $(users[i]).addClass("sayingto");
      } else {
        $(users[i]).removeClass("sayingto");
      }
    }
  }
  function play_ring(url) {
    var embed =
      '<embed id="ring" src="' +
      url +
      '" loop="0" autostart="true" hidden="true" style="height:0px; width:0px;0px;"></embed>';
    $("#ring").html(embed);
  }
  function getTimeShow(time) {
    var dt = new Date(time);
    time =
      dt.getFullYear() +
      "-" +
      (dt.getMonth() + 1) +
      "-" +
      dt.getDate() +
      " " +
      dt.getHours() +
      ":" +
      (dt.getMinutes() < 10 ? "0" + dt.getMinutes() : dt.getMinutes()) +
      ":" +
      (dt.getSeconds() < 10 ? "0" + dt.getSeconds() : dt.getSeconds());
    return time;
  }
  $.cookie("isLogin", true);

  // H5 切换显示用户列表
  $("#user_toggle_btn").click(function () {
    $("#users_online").fadeToggle(200);
  });
});

