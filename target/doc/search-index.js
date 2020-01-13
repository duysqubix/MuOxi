var N=null,E="",T="t",U="u",searchIndex={};
var R=["result","ConnStates","sender","clientdata","to_owned","clone_into","string","try_from","try_into","borrow","borrow_mut","type_id","formatter","on_message","response","ClientData","WebSocketServer","serialize","mongoresult","database","collection","filteron","document","updateoptions","db::clients","db::utils","deserialize","DatabaseHandler","ClientDB","Character","FilterOn","MongoDocument","typeid","clients","db::templates","tcpstream","muoxi_staging","AwaitingAcctName","NewAcctName","MainMenu","EnterGame","connstate","name_mut","option","on_next","mainmenu","on_new_acct","socketaddr","copyover","muoxi_staging::comms","muoxi_staging::connstates","muoxi_staging::copyover","newacctname","entergame","newacct","connstatemessages","awaitingacctname","context","ClientAccount","ConnState","ConnStateMessages","NewAcct","CopyOver","connstates","jsonfile","JsonFile","pgconnection","queryresult","db::schema::clients","predicate","selection","db::schema::clients::columns","execute","client","default","join_target","all_columns","into_update_target","updatetarget","walk_ast","astpass","as_query","FilterCriteria","ClientFilterOn","ClientHandler","Clients","Attempts to insert a new client with UID, if there is a…","ClientHashMap","ClientVector","redisresult","db::cache"];
searchIndex["db"]={"doc":"Diesel powered ORM management library for MuOxi uses…","i":[[3,R[27],"db","Main database handler struct",N,N],[12,"handle",E,"acutal connection to postgres database",0,N],[12,R[33],E,E,0,N],[0,"cache",E,"Wrapper around redis server to easy manipulate cached data",N,N],[3,"Cache",R[90],"main wrapper around redis::Connection",N,N],[12,"conn",E,E,1,N],[7,"REDIS_SERVER",E,"default address for redis server",N,N],[11,"new_with_uri",E,"create new connection to cache server using custom uri",1,[[["str"]],[R[89]]]],[11,"new",E,"create new connection to cache server using default uri",1,[[],[R[89]]]],[0,R[33],"db","Holds templates and query related functions about clients…",N,N],[3,"Client",R[24],E,N,N],[12,"uid",E,E,2,N],[12,"ip",E,E,2,N],[12,"port",E,E,2,N],[3,R[87],E,E,N,N],[12,"0",E,E,3,N],[3,R[88],E,E,N,N],[12,"0",E,E,4,N],[3,R[84],E,E,N,N],[11,"upsert",E,R[86],5,[[[R[66]],["self"],[R[73]]],[[R[67],[R[73]]],[R[73]]]]],[11,"upsert_batch",E,R[86],5,[[[R[66]],["self"],["clientvector"]],[R[67]]]],[11,"remove_uid",E,"Permanently remove record from table, by UID",5,[[["uid"],[R[66]],["self"]],[[R[67],["usize"]],["usize"]]]],[11,"remove_uids",E,"Remove a list of UIDS from db by suppling a vec of UIDs…",5,[[["uid"],[R[66]],["self"],["vec",["uid"]]],[[R[67],["usize"]],["usize"]]]],[11,"get_uid",E,"Get single UID from db",5,[[["uid"],[R[66]],["self"]],[["vec",[R[73]]],[R[67],["vec"]]]]],[11,"get_uids",E,"Retrieve a list of UIDS from db by suppling a vec of UIDs…",5,[[["uid"],[R[66]],["self"],["vec",["uid"]]],[["vec",[R[73]]],[R[67],["vec"]]]]],[11,"get_uids_range",E,"Get a range of UIDs Note that UIDS that do not exists will…",5,[[["uid"],[R[66]],["self"]],[["vec",[R[73]]],[R[67],["vec"]]]]],[11,"get_ip_exact",E,"retrieve records with exact IP address",5,[[[R[66]],["self"],["str"]],[["vec",[R[73]]],[R[67],["vec"]]]]],[11,"get_ip_like",E,"retrieve a vector of ip address with the appropriate match…",5,[[[R[66]],["self"],[R[6]]],[["vec",[R[73]]],[R[67],["vec"]]]]],[0,"schema","db",E,N,N],[0,R[33],"db::schema",E,N,N],[3,"table",R[68],"The actual table struct",N,N],[0,"dsl",E,"Re-exports all of the columns of this table, as well as…",N,N],[0,"columns",E,"Contains all of the columns of this table",N,N],[3,"star",R[71],"Represents `table_name.*`, which is sometimes needed for…",N,N],[3,"uid",E,E,N,N],[3,"ip",E,E,N,N],[3,"port",E,E,N,N],[6,"SqlType",R[68],"The SQL type of all of the columns on this table",N,N],[6,"BoxedQuery",E,"Helper type for representing a boxed query from this table",N,N],[17,R[76],E,"A tuple of all of the columns on this table",N,N],[11,"star",E,"Represents `table_name.*`, which is sometimes necessary…",6,[[["self"]],["star"]]],[0,"templates","db","Holds all the different models and templates that…",N,N],[3,"Account",R[34],E,N,N],[12,"uid",E,E,7,N],[12,"name",E,E,7,N],[0,"utils","db",E,N,N],[5,"gen_uid",R[25],"Creates a unique 8 byte address first 4 bytes is timestamp…",N,[[],["uid"]]],[5,"json_to_object",E,"Attempts to convert serde_json::Value to T",N,[[["value"]],[["deserializeowned"],[R[17]],["jsondecoderresult"]]]],[6,"UID",E,E,N,N],[6,"JsonDecoderResult",E,E,N,N],[11,"connect","db","creates a new instance of the handler and defaults to…",0,[[],["self"]]],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[7],E,E,0,[[[U]],[R[0]]]],[11,R[8],E,E,0,[[],[R[0]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[32]]]],[11,"vzip",E,E,0,[[],["v"]]],[11,"into",R[90],E,1,[[],[U]]],[11,"from",E,E,1,[[[T]],[T]]],[11,R[7],E,E,1,[[[U]],[R[0]]]],[11,R[8],E,E,1,[[],[R[0]]]],[11,R[9],E,E,1,[[["self"]],[T]]],[11,R[10],E,E,1,[[["self"]],[T]]],[11,R[11],E,E,1,[[["self"]],[R[32]]]],[11,"vzip",E,E,1,[[],["v"]]],[11,"into",R[24],E,2,[[],[U]]],[11,"from",E,E,2,[[[T]],[T]]],[11,R[4],E,E,2,[[["self"]],[T]]],[11,R[5],E,E,2,[[["self"],[T]]]],[11,R[7],E,E,2,[[[U]],[R[0]]]],[11,R[8],E,E,2,[[],[R[0]]]],[11,R[9],E,E,2,[[["self"]],[T]]],[11,R[10],E,E,2,[[["self"]],[T]]],[11,R[11],E,E,2,[[["self"]],[R[32]]]],[11,"vzip",E,E,2,[[],["v"]]],[11,"into",E,E,3,[[],[U]]],[11,"from",E,E,3,[[[T]],[T]]],[11,R[7],E,E,3,[[[U]],[R[0]]]],[11,R[8],E,E,3,[[],[R[0]]]],[11,R[9],E,E,3,[[["self"]],[T]]],[11,R[10],E,E,3,[[["self"]],[T]]],[11,R[11],E,E,3,[[["self"]],[R[32]]]],[11,"vzip",E,E,3,[[],["v"]]],[11,"into",E,E,4,[[],[U]]],[11,"from",E,E,4,[[[T]],[T]]],[11,R[7],E,E,4,[[[U]],[R[0]]]],[11,R[8],E,E,4,[[],[R[0]]]],[11,R[9],E,E,4,[[["self"]],[T]]],[11,R[10],E,E,4,[[["self"]],[T]]],[11,R[11],E,E,4,[[["self"]],[R[32]]]],[11,"vzip",E,E,4,[[],["v"]]],[11,"into",E,E,5,[[],[U]]],[11,"from",E,E,5,[[[T]],[T]]],[11,R[7],E,E,5,[[[U]],[R[0]]]],[11,R[8],E,E,5,[[],[R[0]]]],[11,R[9],E,E,5,[[["self"]],[T]]],[11,R[10],E,E,5,[[["self"]],[T]]],[11,R[11],E,E,5,[[["self"]],[R[32]]]],[11,"vzip",E,E,5,[[],["v"]]],[11,"into",R[68],E,6,[[],[U]]],[11,"from",E,E,6,[[[T]],[T]]],[11,R[4],E,E,6,[[["self"]],[T]]],[11,R[5],E,E,6,[[["self"],[T]]]],[11,R[7],E,E,6,[[[U]],[R[0]]]],[11,R[8],E,E,6,[[],[R[0]]]],[11,R[9],E,E,6,[[["self"]],[T]]],[11,R[10],E,E,6,[[["self"]],[T]]],[11,R[11],E,E,6,[[["self"]],[R[32]]]],[11,"filter",E,E,6,[[[R[69]]]]],[11,"group_by",E,E,6,[[["expr"]]]],[11,R[75],E,E,6,[[["onclausewrapper"]]]],[11,R[77],E,E,6,[[],[R[78]]]],[11,"select",E,E,6,[[[R[70]]]]],[11,"internal_into_boxed",E,E,6,[[]]],[11,R[81],E,E,6,[[]]],[11,"internal_load",E,E,6,[[["conn"]],[["error"],["vec"],[R[0],["vec","error"]]]]],[11,"find",E,E,6,[[["pk"]]]],[11,"or_filter",E,E,6,[[[R[69]]]]],[11,"limit",E,E,6,[[["i64"]]]],[11,"for_update",E,E,6,[[]]],[11,"with_lock",E,E,6,[[["lock"]]]],[11,"offset",E,E,6,[[["i64"]]]],[11,"order",E,E,6,[[["expr"]]]],[11,"then_order_by",E,E,6,[[["expr"]]]],[11,"distinct",E,E,6,[[]]],[11,"distinct_on",E,E,6,[[[R[70]]]]],[11,"vzip",E,E,6,[[],["v"]]],[11,"into",R[71],E,8,[[],[U]]],[11,"from",E,E,8,[[[T]],[T]]],[11,R[4],E,E,8,[[["self"]],[T]]],[11,R[5],E,E,8,[[["self"],[T]]]],[11,R[7],E,E,8,[[[U]],[R[0]]]],[11,R[8],E,E,8,[[],[R[0]]]],[11,R[9],E,E,8,[[["self"]],[T]]],[11,R[10],E,E,8,[[["self"]],[T]]],[11,R[11],E,E,8,[[["self"]],[R[32]]]],[11,"vzip",E,E,8,[[],["v"]]],[11,"into",E,E,9,[[],[U]]],[11,"from",E,E,9,[[[T]],[T]]],[11,R[4],E,E,9,[[["self"]],[T]]],[11,R[5],E,E,9,[[["self"],[T]]]],[11,R[7],E,E,9,[[[U]],[R[0]]]],[11,R[8],E,E,9,[[],[R[0]]]],[11,R[9],E,E,9,[[["self"]],[T]]],[11,R[10],E,E,9,[[["self"]],[T]]],[11,R[11],E,E,9,[[["self"]],[R[32]]]],[11,R[72],E,E,9,[[["conn"],[T]],[[R[0],["usize","error"]],["usize"],["error"]]]],[11,"vzip",E,E,9,[[],["v"]]],[11,"into",E,E,10,[[],[U]]],[11,"from",E,E,10,[[[T]],[T]]],[11,R[4],E,E,10,[[["self"]],[T]]],[11,R[5],E,E,10,[[["self"],[T]]]],[11,R[7],E,E,10,[[[U]],[R[0]]]],[11,R[8],E,E,10,[[],[R[0]]]],[11,R[9],E,E,10,[[["self"]],[T]]],[11,R[10],E,E,10,[[["self"]],[T]]],[11,R[11],E,E,10,[[["self"]],[R[32]]]],[11,R[72],E,E,10,[[["conn"],[T]],[[R[0],["usize","error"]],["usize"],["error"]]]],[11,"vzip",E,E,10,[[],["v"]]],[11,"into",E,E,11,[[],[U]]],[11,"from",E,E,11,[[[T]],[T]]],[11,R[4],E,E,11,[[["self"]],[T]]],[11,R[5],E,E,11,[[["self"],[T]]]],[11,R[7],E,E,11,[[[U]],[R[0]]]],[11,R[8],E,E,11,[[],[R[0]]]],[11,R[9],E,E,11,[[["self"]],[T]]],[11,R[10],E,E,11,[[["self"]],[T]]],[11,R[11],E,E,11,[[["self"]],[R[32]]]],[11,R[72],E,E,11,[[["conn"],[T]],[[R[0],["usize","error"]],["usize"],["error"]]]],[11,"vzip",E,E,11,[[],["v"]]],[11,"into",R[34],E,7,[[],[U]]],[11,"from",E,E,7,[[[T]],[T]]],[11,R[7],E,E,7,[[[U]],[R[0]]]],[11,R[8],E,E,7,[[],[R[0]]]],[11,R[9],E,E,7,[[["self"]],[T]]],[11,R[10],E,E,7,[[["self"]],[T]]],[11,R[11],E,E,7,[[["self"]],[R[32]]]],[11,"vzip",E,E,7,[[],["v"]]],[11,"from",R[24],E,4,[[["clienthashmap"]],["self"]]],[11,"clone",E,E,2,[[["self"]],[R[73]]]],[11,"clone",R[68],E,6,[[["self"]],["table"]]],[11,"clone",R[71],E,8,[[["self"]],["star"]]],[11,"clone",E,E,9,[[["self"]],["uid"]]],[11,"clone",E,E,10,[[["self"]],["ip"]]],[11,"clone",E,E,11,[[["self"]],["port"]]],[11,R[74],E,E,9,[[],["uid"]]],[11,R[74],E,E,10,[[],["ip"]]],[11,R[74],E,E,11,[[],["port"]]],[11,"fmt",R[24],E,2,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",R[68],E,6,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",R[71],E,8,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,9,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,10,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,11,[[["self"],[R[12]]],[R[0]]]],[11,"div",E,E,9,[[["rhs"]]]],[11,"div",E,E,11,[[["rhs"]]]],[11,"sub",E,E,9,[[["rhs"]]]],[11,"sub",E,E,11,[[["rhs"]]]],[11,"add",E,E,9,[[["rhs"]]]],[11,"add",E,E,11,[[["rhs"]]]],[11,"mul",E,E,9,[[["rhs"]]]],[11,"mul",E,E,11,[[["rhs"]]]],[11,"table",R[68],E,6,[[]]],[11,"build",R[24],E,2,[[],["self"]]],[11,"build",R[34],E,7,[[],["self"]]],[11,"eq_all",R[71],E,9,[[[T]]]],[11,"eq_all",E,E,10,[[[T]]]],[11,"eq_all",E,E,11,[[[T]]]],[11,"values",R[24],E,2,[[]]],[11,"values",R[68],E,6,[[]]],[11,R[75],E,E,6,[[["join"]]]],[11,R[75],E,E,6,[[["joinon"]]]],[11,R[75],E,E,6,[[["selectstatement"]]]],[11,R[75],E,E,6,[[["boxedselectstatement"]]]],[11,"from_clause",E,E,6,[[["self"]]]],[11,"default_selection",E,E,6,[[["self"]]]],[11,"primary_key",E,E,6,[[["self"]]]],[11,R[76],E,E,6,[[]]],[11,R[77],E,E,6,[[],[R[78]]]],[11,R[79],R[71],E,8,[[["self"],[R[80]]],[R[67]]]],[11,R[79],E,E,9,[[["self"],[R[80]]],[R[67]]]],[11,R[79],E,E,10,[[["self"],[R[80]]],[R[67]]]],[11,R[79],E,E,11,[[["self"],[R[80]]],[R[67]]]],[11,R[81],R[68],E,6,[[]]],[11,"as_changeset",R[24],E,2,[[]]],[11,R[17],E,E,2,[[["self"],["__s"]],[R[0]]]],[11,R[26],E,E,2,[[["__d"]],[R[0]]]]],"p":[[3,R[27]],[3,"Cache"],[3,"Client"],[3,R[87]],[3,R[88]],[3,R[84]],[3,"table"],[3,"Account"],[3,"star"],[3,"uid"],[3,"ip"],[3,"port"]]};
searchIndex["muoxi_engine"]={"doc":"The main MuOxi Game server. This is where all the game…","i":[],"p":[]};
searchIndex["muoxi_sandbox"]={"doc":E,"i":[],"p":[]};
searchIndex["muoxi_staging"]={"doc":"Main Proxy Staging TCP Server","i":[[5,"send",R[36],"Friendly async wrapper to sending messages to client object",N,[[[R[73]],["str"]]]],[5,"get",E,"Friendly async wrapper around recieving message from…",N,[[[R[73]]]]],[5,"process",E,"Main processing piece of logic, once a connection is…",N,[[["arc",["mutex"]],[R[35]],["mutex",["server"]]]]],[5,"transfer",E,"Turns the staging server into a full proxy server,…",N,[[[R[35]],[R[6]]]]],[0,"comms",E,"Definitions and declarations of data structures relating…",N,N],[3,R[58],R[49],"struct holding client account information",N,N],[12,"name",E,E,0,N],[12,"ncharacters",E,E,0,N],[3,"Client",E,"Wrapper around connected socket, this is non-persistent…",N,N],[12,"uid",E,E,1,N],[12,"state",E,E,1,N],[12,"lines",E,E,1,N],[12,"addr",E,E,1,N],[3,"Comms",E,"Server owned structure that holds each clients SocketAddr…",N,N],[12,"0",E,E,2,N],[12,"1",E,E,2,N],[3,"Server",E,"Shared ownership structure between all connected clients.",N,N],[12,R[33],E,"Holds information regarding connected clients",3,N],[12,"accounts",E,"Holds account information for each client",3,N],[4,"Message",E,"The types of message recieved",N,N],[13,"FromClient",E,"Message recieved from connected client",4,N],[13,"OnRx",E,"Message recieved from shared Rx",4,N],[6,"Tx",E,E,N,N],[6,"Rx",E,E,N,N],[11,"new",E,E,0,[[[R[6]]],["self"]]],[11,"new",E,E,1,[[["mutex",["server"]],["uid"],[R[35]],["arc",["mutex"]]]]],[11,"new",E,E,3,[[],["self"]]],[11,"broadcast",E,E,3,[[[R[47]],["str"],["self"]]]],[0,R[48],R[36],"A custom reimplementation of tokio::io::copy",N,N],[3,R[62],R[51],"Asynchronously copies the entire contents of a reader into…",N,N],[5,"copy",E,E,N,[[["w"],["r"],[R[47]]],[["sized"],[R[48]]]]],[7,"GAME_ADDR",R[36],"Current listening port of the MuOxi game engine",N,N],[7,"PROXY_ADDR",E,"Current listening port of the staging proxy server",N,N],[11,"into",R[49],E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[7],E,E,0,[[[U]],[R[0]]]],[11,R[8],E,E,0,[[],[R[0]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[32]]]],[11,"vzip",E,E,0,[[],["v"]]],[11,"into",E,E,1,[[],[U]]],[11,"from",E,E,1,[[[T]],[T]]],[11,R[7],E,E,1,[[[U]],[R[0]]]],[11,R[8],E,E,1,[[],[R[0]]]],[11,R[9],E,E,1,[[["self"]],[T]]],[11,R[10],E,E,1,[[["self"]],[T]]],[11,R[11],E,E,1,[[["self"]],[R[32]]]],[11,"vzip",E,E,1,[[],["v"]]],[11,"try_poll_next",E,E,1,[[[R[57]],["s"],["pin"]],[["poll",[R[43]]],[R[43],[R[0]]]]]],[11,"into",E,E,2,[[],[U]]],[11,"from",E,E,2,[[[T]],[T]]],[11,R[7],E,E,2,[[[U]],[R[0]]]],[11,R[8],E,E,2,[[],[R[0]]]],[11,R[9],E,E,2,[[["self"]],[T]]],[11,R[10],E,E,2,[[["self"]],[T]]],[11,R[11],E,E,2,[[["self"]],[R[32]]]],[11,"vzip",E,E,2,[[],["v"]]],[11,"into",E,E,3,[[],[U]]],[11,"from",E,E,3,[[[T]],[T]]],[11,R[7],E,E,3,[[[U]],[R[0]]]],[11,R[8],E,E,3,[[],[R[0]]]],[11,R[9],E,E,3,[[["self"]],[T]]],[11,R[10],E,E,3,[[["self"]],[T]]],[11,R[11],E,E,3,[[["self"]],[R[32]]]],[11,"vzip",E,E,3,[[],["v"]]],[11,"into",E,E,4,[[],[U]]],[11,"from",E,E,4,[[[T]],[T]]],[11,R[7],E,E,4,[[[U]],[R[0]]]],[11,R[8],E,E,4,[[],[R[0]]]],[11,R[9],E,E,4,[[["self"]],[T]]],[11,R[10],E,E,4,[[["self"]],[T]]],[11,R[11],E,E,4,[[["self"]],[R[32]]]],[11,"vzip",E,E,4,[[],["v"]]],[11,"into",R[51],E,5,[[],[U]]],[11,"from",E,E,5,[[[T]],[T]]],[11,R[7],E,E,5,[[[U]],[R[0]]]],[11,R[8],E,E,5,[[],[R[0]]]],[11,R[9],E,E,5,[[["self"]],[T]]],[11,R[10],E,E,5,[[["self"]],[T]]],[11,R[11],E,E,5,[[["self"]],[R[32]]]],[11,"vzip",E,E,5,[[],["v"]]],[11,"try_poll",E,E,5,[[["pin"],["f"],[R[57]]],["poll"]]],[11,"fmt",R[49],E,4,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,0,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,1,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,2,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,3,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",R[51],E,5,[[["self"],[R[12]]],[R[0]]]],[11,"poll",E,E,5,[[[R[57]],["self"],["pin"]],[["poll",[R[0]]],[R[0],["u64"]]]]],[11,"poll_next",R[49],E,1,[[[R[57]],["self"],["pin"]],[["poll",[R[43]]],[R[43]]]]]],"p":[[3,R[58]],[3,"Client"],[3,"Comms"],[3,"Server"],[4,"Message"],[3,R[62]]]};
searchIndex["muoxi_watchdog"]={"doc":"WatchDog that monitors the custom defined `.json` files…","i":[[4,R[65],"muoxi_watchdog","Different `.json` storage files that need to be monitored",N,N],[13,"Accounts",E,"holds account information ex: number of characters of…",0,N],[13,"Players",E,"holds all character information",0,N],[13,R[85],E,"holds raw socket representation of connected clients",0,N],[5,"read_file",E,"simple wrapper to read from json file and return…",N,[[["str"]],[[R[0],["value"]],["value"]]]],[5,"trigger_upload",E,"main function that triggers upload protocols for each…",N,[[[R[64]]],[[R[0],["box"]],["box",["error"]]]]],[7,"CLIENTS",E,E,N,N],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[4],E,E,0,[[["self"]],[T]]],[11,R[5],E,E,0,[[["self"],[T]]]],[11,R[7],E,E,0,[[[U]],[R[0]]]],[11,R[8],E,E,0,[[],[R[0]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[32]]]],[11,"vzip",E,E,0,[[],["v"]]],[11,"clone",E,E,0,[[["self"]],[R[64]]]],[11,"fmt",E,E,0,[[["self"],[R[12]]],[R[0]]]]],"p":[[4,R[65]]]};
searchIndex["muoxi_web"]={"doc":"Handles all things related to WebSocketServer Like finding…","i":[[3,R[15],"muoxi_web",E,N,N],[12,"ip",E,E,0,N],[12,"token",E,E,0,N],[12,"in_buf",E,E,0,N],[12,"out_buf",E,E,0,N],[3,R[85],E,E,N,N],[12,"client_list",E,E,1,N],[3,R[16],E,E,N,N],[11,"new",E,E,1,[[],["self"]]],[11,"insert",E,E,1,[[[R[2]],["self"],[R[6]]],[R[0]]]],[11,"remove",E,E,1,[[["self"],[R[2]]],[[R[3]],[R[43],[R[3]]]]]],[11,"new",E,E,2,[[[R[2]],["mutex",[R[33]]],["arc",["mutex"]]],["self"]]],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[4],E,E,0,[[["self"]],[T]]],[11,R[5],E,E,0,[[["self"],[T]]]],[11,R[7],E,E,0,[[[U]],[R[0]]]],[11,R[8],E,E,0,[[],[R[0]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[32]]]],[11,"vzip",E,E,0,[[],["v"]]],[11,"into",E,E,1,[[],[U]]],[11,"from",E,E,1,[[[T]],[T]]],[11,R[4],E,E,1,[[["self"]],[T]]],[11,R[5],E,E,1,[[["self"],[T]]]],[11,"to_string",E,E,1,[[["self"]],[R[6]]]],[11,R[7],E,E,1,[[[U]],[R[0]]]],[11,R[8],E,E,1,[[],[R[0]]]],[11,R[9],E,E,1,[[["self"]],[T]]],[11,R[10],E,E,1,[[["self"]],[T]]],[11,R[11],E,E,1,[[["self"]],[R[32]]]],[11,"vzip",E,E,1,[[],["v"]]],[11,"into",E,E,2,[[],[U]]],[11,"from",E,E,2,[[[T]],[T]]],[11,R[7],E,E,2,[[[U]],[R[0]]]],[11,R[8],E,E,2,[[],[R[0]]]],[11,R[9],E,E,2,[[["self"]],[T]]],[11,R[10],E,E,2,[[["self"]],[T]]],[11,R[11],E,E,2,[[["self"]],[R[32]]]],[11,R[13],E,E,2,[[["message"],["self"]],[[R[0],["error"]],["error"]]]],[11,"vzip",E,E,2,[[],["v"]]],[11,"clone",E,E,0,[[["self"]],[R[3]]]],[11,"clone",E,E,1,[[["self"]],[R[33]]]],[11,"eq",E,E,0,[[[R[3]],["self"]],["bool"]]],[11,"ne",E,E,0,[[[R[3]],["self"]],["bool"]]],[11,"fmt",E,E,1,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,0,[[["self"],[R[12]]],[R[0]]]],[11,"fmt",E,E,1,[[["self"],[R[12]]],[R[0]]]],[11,"hash",E,E,0,[[["self"],["__h"]]]],[11,R[13],E,E,2,[[["self"],["message"]],[R[0]]]],[11,"on_open",E,E,2,[[["handshake"],["self"]],[R[0]]]],[11,"on_request",E,E,2,[[["self"],["request"]],[[R[14]],[R[0],[R[14]]]]]],[11,"on_close",E,E,2,[[["str"],["self"],["closecode"]]]]],"p":[[3,R[15]],[3,R[85]],[3,R[16]]]};
searchIndex["states"]={"doc":E,"i":[[4,R[1],"states",E,N,N],[13,"AwaitingName",E,E,0,N],[13,"AwaitingPassword",E,E,0,N],[13,"AwaitingNewName",E,E,0,N],[13,"AwaitingNewPassword",E,E,0,N],[13,"ConfirmNewPassword",E,E,0,N],[13,R[39],E,E,0,N],[13,"Playing",E,E,0,N],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[4],E,E,0,[[["self"]],[T]]],[11,R[5],E,E,0,[[["self"],[T]]]],[11,R[7],E,E,0,[[[U]],[R[0]]]],[11,R[8],E,E,0,[[],[R[0]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,R[10],E,E,0,[[["self"]],[T]]],[11,R[11],E,E,0,[[["self"]],[R[32]]]],[11,"clone",E,E,0,[[["self"]],[R[63]]]],[11,"fmt",E,E,0,[[["self"],[R[12]]],[R[0]]]],[11,R[17],E,E,0,[[["self"],["__s"]],[R[0]]]],[11,R[26],E,E,0,[[["__d"]],[R[0]]]]],"p":[[4,R[1]]]};
initSearch(searchIndex);addSearchOptions(searchIndex);